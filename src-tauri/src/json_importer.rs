use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::time::Instant;
use neo4rs::{Graph, query, ConfigBuilder};
use tauri::{Window, Emitter};
use zip::read::ZipArchive;

// Builtin SIDs of identities that should be imported
const BUILTIN_SIDS_QUERY: &str = r#"
UNWIND [
    {sid: "S-1-5-18", name: "NT AUTHORITY\\SYSTEM"},
    {sid: "S-1-5-19", name: "NT AUTHORITY\\LOCAL SERVICE"},
    {sid: "S-1-5-20", name: "NT AUTHORITY\\NETWORK SERVICE"},
    {sid: "S-1-5-32-544", name: "BUILTIN\\Administrators"},
    {sid: "S-1-5-32-545", name: "BUILTIN\\Users"},
    {sid: "S-1-5-32-546", name: "BUILTIN\\Guests"},
    {sid: "S-1-5-32-547", name: "BUILTIN\\Power Users"},
    {sid: "S-1-5-32-548", name: "BUILTIN\\Account Operators"},
    {sid: "S-1-5-32-549", name: "BUILTIN\\Server Operators"},
    {sid: "S-1-5-32-550", name: "BUILTIN\\Print Operators"},
    {sid: "S-1-5-32-551", name: "BUILTIN\\Backup Operators"},
    {sid: "S-1-5-32-552", name: "BUILTIN\\Replicators"},
    {sid: "S-1-5-32-554", name: "BUILTIN\\Pre-Windows 2000 Compatible Access"},
    {sid: "S-1-5-32-555", name: "BUILTIN\\Remote Desktop Users"},
    {sid: "S-1-5-32-556", name: "BUILTIN\\Network Configuration Operators"},
    {sid: "S-1-5-32-557", name: "BUILTIN\\Incoming Forest Trust Builders"},
    {sid: "S-1-5-32-558", name: "BUILTIN\\Performance Monitor Users"},
    {sid: "S-1-5-32-559", name: "BUILTIN\\Performance Log Users"},
    {sid: "S-1-5-32-560", name: "BUILTIN\\Windows Authorization Access Group"},
    {sid: "S-1-5-32-561", name: "BUILTIN\\Terminal Server License Servers"},
    {sid: "S-1-5-32-562", name: "BUILTIN\\Distributed COM Users"},
    {sid: "S-1-5-32-568", name: "BUILTIN\\IIS_IUSRS"},
    {sid: "S-1-5-32-569", name: "BUILTIN\\Cryptographic Operators"},
    {sid: "S-1-5-32-573", name: "BUILTIN\\Event Log Readers"},
    {sid: "S-1-5-32-574", name: "BUILTIN\\Certificate Service DCOM Access"},
    {sid: "S-1-5-32-575", name: "BUILTIN\\RDS Remote Access Servers"},
    {sid: "S-1-5-32-576", name: "BUILTIN\\RDS Endpoint Servers"},
    {sid: "S-1-5-32-577", name: "BUILTIN\\RDS Management Servers"},
    {sid: "S-1-5-32-578", name: "BUILTIN\\Hyper-V Administrators"},
    {sid: "S-1-5-32-579", name: "BUILTIN\\Access Control Assistance Operators"},
    {sid: "S-1-5-32-580", name: "BUILTIN\\Remote Management Users"},
    {sid: "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464", name: "NT SERVICE\\TrustedInstaller"},
    {sid: "S-1-5-32-583", name: "BUILTIN\\Device Owners"},
    {sid: "S-1-15-2-1", name: "ALL APPLICATION PACKAGES"},
    {sid: "S-1-15-2-2", name: "ALL RESTRICTED APPLICATION PACKAGES"},
    {sid: "S-1-1-0", name: "Everyone"},
    {sid: "S-1-5-11", name: "Authenticated Users"},
    {sid: "S-1-5-2", name: "NETWORK"},
    {sid: "S-1-5-4", name: "INTERACTIVE"},
    {sid: "S-1-5-6", name: "SERVICE"},
    {sid: "S-1-5-7", name: "ANONYMOUS"},
    {sid: "S-1-5-9", name: "ENTERPRISE DOMAIN CONTROLLERS"},
    {sid: "S-1-5-10", name: "Principal Self"},
    {sid: "S-1-3-0", name: "CREATOR OWNER"},
    {sid: "S-1-3-1", name: "CREATOR GROUP"},
    {sid: "S-1-5-32-553", name: "BUILTIN\\Backup Operators"}
] AS item
MERGE (n:Identity {sid: item.sid})
SET n.name = item.name
"#;

//Query to Run after successful Import to link users to some builtin Groups. Sid -513 is the sid for Domain users group -> SID and Name depend on the domain.
const LINK_USERS_TO_BUILTIN_GROUPS_QUERY: &str = r#"
MATCH (u:User)
MATCH (i:Identity)
WHERE i.name IN ["Everyone", "Authenticated Users"]
OR i.sid ENDS WITH "-513"
MERGE (u)-[:MEMBER_OF]->(i)"#;

//Add Domain Group to Builtin users group. Needed to Fix the Issue we have when querying ldap for the sid of builtin groups. this query returns no sid so name is set. on english OS this is fixed on query, fo others we need this. See active_index_ldap.rs (Line 295) for more details.
const ADD_DOMAIN_GROUP_TO_BUILTIN_USERS_GROUP_QUERY: &str = r#"
MATCH (o:Identity)
WHERE o.sid ENDS WITH "-513"
MATCH (b:Identity)
WHERE b.sid = "S-1-5-32-545"
MERGE (o)-[:MEMBER_OF]->(b)
"#;

//Add index for fullpath property on files. speeds up queries for files.
const ADD_INDEX_FOR_FULLPATH_ON_FILES_QUERY: &str = r#"
CREATE INDEX IF NOT EXISTS FOR (f:file) ON (f.full_path);
"#;

const ADD_INDEX_FOR_IDENTITY_SID_QUERY: &str = r#"
CREATE INDEX IF NOT EXISTS FOR (i:Identity) ON (i.sid);
"#;

#[derive(Serialize, Deserialize, Clone)]
pub struct ImportStats {
    pub nodes_imported: usize,
    pub elapsed_seconds: f64,
    pub batches_processed: usize,
    pub total_batches: usize,
    pub percentage_complete: f32,
}

#[derive(Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub is_jsonl: bool,
}

#[tauri::command]
pub async fn import_json_to_neo4j(
    window: Window,
    file_path: String,
    neo4j_uri: String,
    neo4j_user: String,
    neo4j_password: String,
    batch_size: usize,
    import_mode: String,
    is_initial_share_import: bool,
) -> Result<ImportStats, String> {
    let result = import_large_json(
        window,
        file_path,
        neo4j_uri,
        neo4j_user,
        neo4j_password,
        batch_size,
        import_mode,
        is_initial_share_import,
    )
    .await
    .map_err(|e| e.to_string())?;
    
    println!("Returning to JavaScript: {{ nodes_imported: {}, elapsed_seconds: {:.2}, batches: {}/{} }}",
             result.nodes_imported,
             result.elapsed_seconds,
             result.batches_processed,
             result.total_batches);
    
    Ok(result)
}


#[tauri::command]
pub fn get_file_info(file_path: String) -> Result<FileMetadata, String> {
    // Get file metadata only, without reading contents
    let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();
    
    // Check if file has .jsonl extension
    let is_jsonl = file_path.to_lowercase().ends_with(".jsonl");
    
    Ok(FileMetadata {
        size: file_size,
        is_jsonl,
    })
}

pub async fn import_large_json(
    window: Window,
    file_path: String,
    neo4j_uri: String,
    neo4j_user: String,
    neo4j_password: String,
    batch_size: usize,
    import_mode: String,
    is_initial_share_import: bool,
) -> Result<ImportStats, Box<dyn std::error::Error>> {
    // Initialize stats
    let mut stats = ImportStats {
        nodes_imported: 0,
        elapsed_seconds: 0.0,
        batches_processed: 0,
        total_batches: 0,
        percentage_complete: 0.0,
    };
    
    let start_time = Instant::now();
    
    // Check file format
    let is_jsonl = file_path.to_lowercase().ends_with(".jsonl");
    let is_zip = file_path.to_lowercase().ends_with(".zip");
    
    // Create Neo4j connection
    let graph = create_neo4j_connection(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    
    // Validate the import mode before proceeding
    if !["users", "groups", "computers", "domain", "shares"].contains(&import_mode.as_str()) {
        return Err(format!("Invalid import mode: {}", import_mode).into());
    }
    
    if is_jsonl {
        // Process JSONL file (line by line)
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
        
        stats.total_batches = (lines.len() + batch_size - 1) / batch_size; // Ceiling division
        
        // Process lines in batches
        for (batch_index, chunk) in lines.chunks(batch_size).enumerate() {
            // Update batch progress
            stats.batches_processed = batch_index + 1;
            stats.percentage_complete = (stats.batches_processed as f32 / stats.total_batches as f32) * 100.0;
            
            // Send progress update
            emit_progress(&window, &stats);
            
            // Parse each line into a JSON Value
            let mut json_objects = Vec::with_capacity(chunk.len());
            for line in chunk {
                let value: Value = from_str(line)?;
                json_objects.push(value);
            }
            
            // Process batch with the appropriate mode
            let imported = process_batch(&graph, &json_objects, &import_mode, is_initial_share_import).await?;
            stats.nodes_imported += imported;
            println!("Running total: {} nodes imported so far", stats.nodes_imported);
        }
        //RUN EXTRA QUERIES AFTER IMPORT
       
        //create index for fullpath on files
        println!("Creating index for fullpath on files");
        let mut result = graph.execute(query(ADD_INDEX_FOR_FULLPATH_ON_FILES_QUERY)).await?;
        if let Some(_) = result.next().await? {
            println!("Index for fullpath on files created successfully");
        }
        //create index for identity sid
        println!("Creating index for identity sid");
        let mut result = graph.execute(query(ADD_INDEX_FOR_IDENTITY_SID_QUERY)).await?;
        if let Some(_) = result.next().await? {
            println!("Index for identity sid created successfully");
        }

    } else if is_zip && import_mode == "domain" {
        // Process ZIP file for domain mode
        let file = File::open(&file_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Track total files for progress
        let total_files = archive.len();
        
        // First scan the archive to find the files we want to process
        let mut computers_index = None;
        let mut users_index = None;
        let mut groups_index = None;
        
        // Find all the relevant files first
        for i in 0..archive.len() {
            let zip_file = archive.by_index(i)?;
            let name = zip_file.name().to_string();
            
            if name.starts_with("computers_") {
                computers_index = Some(i);
            } else if name.starts_with("users_") {
                users_index = Some(i);
            } else if name.starts_with("groups_") {
                groups_index = Some(i);
            }
        }
        
        // Process in the desired order: computers, then users, then groups
        let mut files_processed = 0;
        let processing_order = [
            ("computers", computers_index),
            ("users", users_index),
            ("groups", groups_index)
        ];
        
        for (mode, maybe_index) in processing_order.iter() {
            if let Some(index) = maybe_index {
                let mut zip_file = archive.by_index(*index)?;
                let name = zip_file.name().to_string();
                println!("Processing {} file from zip: {}", mode, name);
                
                // Read contents into a string
                let mut contents = String::new();
                zip_file.read_to_string(&mut contents)?;
                
                // Process content line by line
                let reader = BufReader::new(contents.as_bytes());
                let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
                
                stats.total_batches = (lines.len() + batch_size - 1) / batch_size; // Ceiling division
                
                // Process lines in batches
                for (batch_index, chunk) in lines.chunks(batch_size).enumerate() {
                    // Update batch progress
                    stats.batches_processed = batch_index + 1;
                    stats.percentage_complete = 
                        ((files_processed as f32 / total_files as f32) + 
                         (stats.batches_processed as f32 / stats.total_batches as f32 / total_files as f32)) * 100.0;
                    
                    // Send progress update
                    emit_progress(&window, &stats);
                    
                    // Parse each line into a JSON Value
                    let mut json_objects = Vec::with_capacity(chunk.len());
                    for line in chunk {
                        let value: Value = from_str(line)?;
                        json_objects.push(value);
                    }
                    
                    // Process batch with the specific mode
                    let imported = process_batch(&graph, &json_objects, mode, false).await?;
                    stats.nodes_imported += imported;
                    println!("Running total: {} nodes imported so far", stats.nodes_imported);
                }
                
                files_processed += 1;
            }
        }
        
        //RUN EXTRA QUERIES AFTER IMPORT

        // Execute the builtin SIDs query after domain data import is complete
         println!("Domain import complete, now importing builtin SIDs");
         let mut result = graph.execute(query(BUILTIN_SIDS_QUERY)).await?;
            if let Some(_) = result.next().await? {
              println!("Builtin SIDs imported successfully");
            }
        
        // Execute the link users to builtin groups query after domain data import is complete
        println!("Linking users to builtin groups");
        let mut result = graph.execute(query(LINK_USERS_TO_BUILTIN_GROUPS_QUERY)).await?;
        if let Some(_) = result.next().await? {
            println!("Users linked to builtin groups successfully");
        }
    
        //Execute ADD_DOMAIN_GROUP_TO_BUILTIN_USERS_GROUP_QUERY -- fix for issue with builtin groups not having a sid.
        println!("Adding domain group to builtin users group");
        let mut result = graph.execute(query(ADD_DOMAIN_GROUP_TO_BUILTIN_USERS_GROUP_QUERY)).await?;
        if let Some(_) = result.next().await? {
            println!("Domain group added to builtin users group successfully");
        }

        //create index for identity sid
        println!("Creating index for identity sid");
        let mut result = graph.execute(query(ADD_INDEX_FOR_IDENTITY_SID_QUERY)).await?;
        if let Some(_) = result.next().await? {
            println!("Index for identity sid created successfully");
        }
        
        
    } else {
        return Err(format!("Invalid file format or mode combination: {} file with {} mode", 
                         if is_jsonl { "JSONL" } else if is_zip { "ZIP" } else { "Unknown" }, 
                         import_mode).into());
    }
    
    // Calculate elapsed time and ensure it's a valid f64
    let elapsed = start_time.elapsed();
    stats.elapsed_seconds = elapsed.as_secs_f64();
    
    // Debug print the stats before returning
    println!("Import completed: {} nodes, {:.2}s, {} batches", 
             stats.nodes_imported, stats.elapsed_seconds, stats.batches_processed);
    
    // Send progress update
    emit_progress(&window, &stats);
    
    Ok(stats)
}

async fn create_neo4j_connection(uri: &str, username: &str, password: &str) 
    -> Result<Graph, Box<dyn std::error::Error>> {
    let config = ConfigBuilder::default()
        .uri(uri)
        .user(username)
        .password(password)
        .build()?;
    
    let graph = Graph::connect(config).await?;
    Ok(graph)
}

async fn process_batch(graph: &Graph, chunk: &[Value], mode: &str, is_initial_share_import: bool) -> Result<usize, Box<dyn std::error::Error>> {
    let query_text = get_import_query_for_mode(mode, is_initial_share_import);
    
    // Convert the chunk to a JSON string (which neo4rs can handle)
    let data_json = serde_json::to_string(chunk)?;
    
    // Print debug info about the batch being processed
    println!("Processing batch of {} items with mode: {}", chunk.len(), mode);
    
    // Create the query with a parameter neo4rs can handle (string)
    let mut result = graph.execute(query(&query_text).param("data", data_json)).await?;
    
    // Try to extract count if available in query result
    let count = if let Some(row) = result.next().await? {
        println!("Got result row - trying to extract count");
        
        if let Ok(value) = row.get::<i64>("count") {
            println!("Successfully extracted count: {}", value);
            value as usize
        } else {
            println!("Failed to extract count, using chunk length: {}", chunk.len());
            // If no count is returned, use chunk length
            chunk.len()
        }
    } else {
        println!("No result row returned, using chunk length: {}", chunk.len());
        chunk.len()
    };
    
    println!("Batch imported {} nodes", count);
    Ok(count)
}

// Get the appropriate Cypher query based on import mode
fn get_import_query_for_mode(mode: &str, is_initial_share_import: bool) -> String {
    match mode {
        "users" => {
            String::from("
            WITH apoc.convert.fromJsonList($data) AS items
            UNWIND items AS item
            MERGE (i:Identity {sid: item.sid})
            SET i:User,
                i.distinguished_name = item.distinguished_name,
                i.name = item.cn,
                i.sam_account_name = item.sam_account_name,
                i.when_created = item.when_created,
                i.last_logon = datetime(item.last_logon)
            RETURN count(i) as count")
        },
        "groups" => {
            String::from("
            WITH apoc.convert.fromJsonList($data) AS items
                UNWIND items AS item

                // Merge group as an Identity node and label it also as Group
                MERGE (i:Identity {sid: item.sid})
                SET i:Group,
                    i.name = item.cn,
                    i.distinguished_name = item.distinguished_name

                WITH i, item.members AS members
                UNWIND members AS member_dn
                MATCH (m:Identity {distinguished_name: member_dn})
                MERGE (m)-[:MEMBER_OF]->(i)

            RETURN count(i) as count;")
        },
        "shares" => {
            if is_initial_share_import {
                String::from("
                WITH apoc.convert.fromJsonList($data) AS items
                UNWIND items AS item

                    // Create the entry node (no duplicate checks)
                    CREATE (entry { full_path: item.full_path })
                    REMOVE entry:Entry
                    SET entry.name = item.name,
                        entry.size = item.size,
                        entry.extension = item.extension,
                        entry.created = item.created,
                        entry.modified = item.modified

                    // Dynamically label the node
                    WITH entry, item
                    CALL apoc.create.setLabels(entry, [item.entry_type]) YIELD node
                    WITH node AS entry, item

                    // Unwind the ACL array
                    WITH entry, item
                    UNWIND CASE WHEN item.acls IS NOT NULL THEN item.acls ELSE [] END AS acl
                        // Merge the Identity node (duplicate checks)
                        MERGE (identity:Identity { sid: acl.identity })

                        // Important: add a WITH here before the next UNWIND
                        WITH entry, identity, acl

                        // Unwind permissions to create dynamic relationships
                        UNWIND acl.permissions AS permission
                        CALL apoc.create.relationship(
                            identity,
                            permission,
                            { ace_type: acl.ace_type, access_mask: acl.access_mask },
                            entry
                        ) YIELD rel

                RETURN count(entry) AS count")
            } else {    
                //Add same query as non intitial mode but use merge and not create
                String::from("
                WITH apoc.convert.fromJsonList($data) AS items
                UNWIND items AS item
                // Match any existing file, directory, or share node with the same full_path
                OPTIONAL MATCH (existing)
                WHERE (existing:file OR existing:directory OR existing:share)
                AND existing.full_path = item.full_path
                WITH item, existing
                CALL apoc.do.when(
                    existing IS NOT NULL,
                    'RETURN existing AS entry',
                    'CREATE (entry:Entry { full_path: item.full_path }) RETURN entry',
                    {item: item, existing: existing}
                ) YIELD value
                WITH value.entry AS entry, item
                SET entry.name = item.name,
                    entry.size = item.size,
                    entry.extension = item.extension,
                    entry.created = item.created,
                    entry.modified = item.modified


                    // Dynamically label the node only if a new node was created with the merge 
                    WITH entry, item
                    CALL apoc.create.setLabels(entry, [item.entry_type]) YIELD node
                    WITH node AS entry, item

                    // Unwind the ACL array
                    WITH entry, item
                    UNWIND CASE WHEN item.acls IS NOT NULL THEN item.acls ELSE [] END AS acl
                        // Merge the Identity node (duplicate checks)
                        MERGE (identity:Identity { sid: acl.identity })

                        // Important: add a WITH here before the next UNWIND
                        WITH entry, identity, acl

                        // Unwind permissions to create dynamic relationships
                        UNWIND acl.permissions AS permission
                        CALL apoc.create.relationship(
                            identity,
                            permission,
                            { ace_type: acl.ace_type, access_mask: acl.access_mask },
                            entry
                        ) YIELD rel

                RETURN count(entry) AS count")
            }
        },
        "computers" => {            
              String::from("
              WITH apoc.convert.fromJsonList($data) AS items
              UNWIND items AS item
              CREATE (n:Computer {distinguished_name: item.distinguished_name})
              SET n.name = item.cn,
                  n.dns_hostname = item.dns_hostname,
                  n.operating_system = item.operating_system,
                  n.os_version = item.os_version,
                  n.when_created = item.when_created,
                  n.last_logon = datetime(item.last_logon)
              RETURN count(n) as count")
        }
        _ => {
            String::from("Error: Unknown mode")
        }
    }
}

// Function to emit progress events
fn emit_progress(window: &Window, stats: &ImportStats) {
    // Print debug info about the stats being sent
    println!(
        "Emitting progress: {} nodes, {} batches processed, {} total batches, {:.2}% complete", 
        stats.nodes_imported, 
        stats.batches_processed, 
        stats.total_batches, 
        stats.percentage_complete
    );
    
    window.emit("import-progress", stats.clone()).unwrap_or_else(|e| {
        eprintln!("Failed to emit progress event: {}", e);
    });
} 