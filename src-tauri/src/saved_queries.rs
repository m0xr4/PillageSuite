use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Query {
    pub name: String,
    pub cypher: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueriesConfig {
    pub predefined: Vec<Query>,
    pub identity: Vec<Query>,
    pub user: Vec<Query>,
}

impl Default for QueriesConfig {
    fn default() -> Self {
        Self {
            predefined: vec![
                Query {
                    name: "Search Password Files".to_string(),
                    cypher: "MATCH (a:file where toLower(a.name) =~ '.*passw.*|.*unattend.*|.*zugang.*|.*login.*|.*zugriff.*|.*credential.*') RETURN a.name as Name,a.full_path as Path,a.extension as Extension".to_string(),
                },
                Query {
                    name: "Search bak Files".to_string(),
                    cypher: "MATCH (a:file where a.extension = 'bak') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search Scripts".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(sql|cmd|bat|ps1|vbs|hta)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search Key/Cert Files".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(key|crt|cer|pfx|p12|pem|csr|p7b|p7c|p7r|p7s)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search Config Files".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(cfg|conf|ini|xml|json|yaml|yml|properties|toml|hocon|env|sh|bash|zsh|config|tcsh|csh)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search VM Files".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(vmx|vmdk|vmsd|vmsn|vmss|vmxf|vmxh|vmxw|vhdx|vhdi|vhd|vhdx|vdi|vdi)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search Webroots".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(aspx|asp|php|jsp|html|htm)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search Common Documents".to_string(),
                    cypher: "MATCH (a:file where a.extension =~ '^(doc|docx|xls|xlsx|ppt|pptx|pdf)$') RETURN a.name as Name,a.size as Size,a.extension as Extension,a.full_path as Path".to_string(),
                },
                Query {
                    name: "Interesting Paths to Shares for low privilege groups".to_string(),
                    cypher: "MATCH p=(i:Identity WHERE i.name IN ['Everyone', 'Authenticated Users', 'BUILTIN\\Users'] OR i.sid contains '-513')-[r]-(s:share) where type(r) IN ['FullControl', 'ReadData/ListDirectory', 'WriteData/AddFile'] RETURN p".to_string(),
                },
                Query {
                    name: "Get ACLs of low privilege groups to Shares".to_string(),
                    cypher: "MATCH (i:Identity)-[r]-(s:share) WHERE i.name IN ['Everyone', 'Authenticated Users', 'BUILTIN\\Users'] OR i.sid contains '-513' RETURN s.name AS ShareName, i.name AS Identity, collect(type(r)) AS Permissions ORDER BY ShareName, Identity".to_string(),
                },
                Query {
                    name: "Search POS System Files".to_string(),
                    cypher: "MATCH (a:file) WHERE a.extension =~ '^(csv|txt|log|db|mdb|sqlite|pos|rpt|json|xml)$' AND toLower(a.name) =~ '.*pos.*|.*receipt.*|.*transaction.*|.*payment.*|.*terminal.*|.*checkout.*|.*invoice.*|.*retail.*|.*sale.*|.*register.*|.*lightspeed.*|.*square.*|.*shopify.*|.*clover.*|.*aloha.*|.*faktur.*|.*x3000.*|.*kasse.*' RETURN a.name as Name, a.size as Size, a.extension as Extension, a.full_path as Path".to_string(),
                },
                Query {
                    name: "Search for PII Documents".to_string(),
                    cypher: "MATCH (a:file) WHERE a.extension IN ['pdf', 'xls', 'xlsx'] AND toLower(a.name) =~ '.*ausweis.*|.*reisepass.*|.*passport.*|.*versicherung.*|.*AHV.*|.*insurance.*|.*steuer.*|.*tax.*|.*gehalt.*|.*salary.*|.*vertrag.*|.*contract.*|.*lebenslauf.*|.*resume.*|.*cv.*|.*zeugnis.*|.*certificate.*|.*bescheinigung.*|.*bewohner.*|.*license.*|.*identification.*|.*id.*|.*ssn.*|.*sozial.*|.*payroll.*|.*lohn.*|.*medical.*|.*bank.*|.*employee.*|.*mitarbeiter.*|.*kreditkarte.*|.*privat.*|.*vertraulich.*|.*confidential.*' RETURN a.name as Name, a.size as Size, a.extension as Extension, a.full_path as Path".to_string(),
                },
            ],
            identity: vec![
                // Identity-specific predefined queries
                Query {
                    name: "Path to Password Files".to_string(),
                    cypher: "MATCH p=(i:Identity WHERE i.sid = $nodeSID)-[*]->(a:file) 
    WHERE toLower(a.name) =~ '.*passw.*|.*unattend.*|.*zugang.*|.*login.*|.*zugriff.*|.*credential.*' 
    AND any(r IN relationships(p) WHERE type(r) IN ['ReadData/ListDirectory', 'Read', 'ReadAndExecute', 'ReadAndWrite', 'Modify', 'FullControl', 'GenericRead', 'GenericAll'])
    RETURN p".to_string(),
                },
                Query {
                    name: "Write Access to Executables".to_string(),
                    cypher: "
                        MATCH p=(i:Identity WHERE i.sid = $nodeSID)-[*]->(a:file)
                        WHERE a.extension IN ['exe', 'dll', 'bat', 'cmd', 'ps1', 'vbs', 'hta', 'msi', 'com', 'scr', 'cpl']
                        AND any(r IN relationships(p) WHERE type(r) IN ['WriteData/AddFile', 'AppendData/AddSubdirectory', 'WriteExtendedAttributes', 'WriteAttributes', 'WriteOwner', 'WriteDAC', 'FullControl', 'Modify', 'ReadAndWrite', 'Write', 'GenericWrite', 'GenericAll'])
                        RETURN p
                    ".to_string(),
                },
                Query {
                    name: "Find Accessible Shares".to_string(),
                    cypher: "
                    MATCH p=(i:Identity WHERE i.sid = $nodeSID)-[*]->(a:share) 
                    WHERE any(r IN relationships(p) WHERE type(r) IN ['ReadData/ListDirectory', 'Read', 'ReadAndExecute', 'ReadAndWrite', 'Modify', 'FullControl', 'GenericRead', 'GenericAll'])
                    RETURN p".to_string(),
                },
                Query {
                    name: "Show associated Identities".to_string(),
                    cypher: "
                    MATCH p=(i:Identity WHERE i.sid = $nodeSID)-[:MEMBER_OF]-(a:Identity)
                    RETURN p
                    ".to_string(),
                },
                
            ],
            user: Vec::new(),
        }
    }
}

fn get_config_dir(app: &AppHandle) -> PathBuf {
    let app_dir = app
        .path()
        .app_config_dir()
        .expect("Failed to get app config directory");
    
    println!("Config directory path: {:?}", app_dir);
    
    // Create the directory if it doesn't exist
    if !app_dir.exists() {
        println!("Creating config directory as it doesn't exist");
        fs::create_dir_all(&app_dir).expect("Failed to create app config directory");
    } else {
        println!("Config directory already exists");
    }
    
    app_dir
}

fn get_queries_path(app: &AppHandle) -> PathBuf {
    let path = get_config_dir(app).join("saved_queries.json");
    println!("Queries file path: {:?}", path);
    path
}

fn load_queries_from_file(path: &Path) -> QueriesConfig {
    println!("Loading queries from: {:?}", path);
    if path.exists() {
        println!("File exists, reading content");
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    println!("Successfully parsed config");
                    config
                },
                Err(e) => {
                    eprintln!("Error parsing queries file: {}", e);
                    println!("Using default config due to parse error");
                    QueriesConfig::default()
                }
            },
            Err(e) => {
                eprintln!("Error reading queries file: {}", e);
                println!("Using default config due to read error");
                QueriesConfig::default()
            }
        }
    } else {
        println!("File doesn't exist, creating default config");
        // Create default config
        let default_config = QueriesConfig::default();
        
        // Save default config to file
        if let Ok(content) = serde_json::to_string_pretty(&default_config) {
            println!("Writing default config to: {:?}", path);
            if let Err(e) = fs::write(path, content) {
                eprintln!("Error writing default queries file: {}", e);
                println!("Failed to write default config");
            } else {
                println!("Successfully wrote default config");
            }
        }
        
        default_config
    }
}

#[tauri::command]
pub fn get_saved_queries(app: AppHandle) -> Result<QueriesConfig, String> {
    println!("Called get_saved_queries");
    let path = get_queries_path(&app);
    Ok(load_queries_from_file(&path))
}

#[tauri::command]
pub fn add_user_query(
    app: AppHandle,
    name: String,
    cypher: String,
) -> Result<QueriesConfig, String> {
    println!("Called add_user_query with name: {}", name);
    let path = get_queries_path(&app);
    let mut config = load_queries_from_file(&path);
    
    config.user.push(Query { name, cypher });
    
    println!("Saving updated config with new query");
    save_queries_to_file(&path, &config)?;
    
    Ok(config)
}

#[tauri::command]
pub fn delete_user_query(app: AppHandle, index: usize) -> Result<QueriesConfig, String> {
    println!("Called delete_user_query with index: {}", index);
    let path = get_queries_path(&app);
    let mut config = load_queries_from_file(&path);
    
    if index < config.user.len() {
        println!("Removing query at index: {}", index);
        config.user.remove(index);
        save_queries_to_file(&path, &config)?;
    } else {
        println!("Index out of bounds: {}, length: {}", index, config.user.len());
    }
    
    Ok(config)
}

fn save_queries_to_file(path: &Path, config: &QueriesConfig) -> Result<(), String> {
    println!("Saving queries to: {:?}", path);
    match serde_json::to_string_pretty(config) {
        Ok(content) => match fs::write(path, content) {
            Ok(_) => {
                println!("Successfully wrote config to file");
                Ok(())
            },
            Err(e) => {
                let error = format!("Error writing queries file: {}", e);
                eprintln!("{}", error);
                Err(error)
            },
        },
        Err(e) => {
            let error = format!("Error serializing queries: {}", e);
            eprintln!("{}", error);
            Err(error)
        },
    }
} 