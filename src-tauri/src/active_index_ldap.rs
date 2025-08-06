use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry, LdapError};
use ldap3::adapters::{Adapter, EntriesOnly, PagedResults};
use serde::{Serialize, Deserialize};
use serde_json::to_writer;
use std::fs::{File, remove_file};
use std::io::{BufWriter, Write, BufReader};
use std::path::Path;
use chrono::{DateTime, Utc};
use tauri::{Window, Emitter};
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

#[derive(Debug)]
enum EnumError {
    Ldap(LdapError),
    Json(()),
    Io(()),
    Zip(()),
}

impl From<LdapError> for EnumError {
    fn from(error: LdapError) -> Self {
        EnumError::Ldap(error)
    }
}

impl From<serde_json::Error> for EnumError {
    fn from(_error: serde_json::Error) -> Self {
        EnumError::Json(())
    }
}

impl From<std::io::Error> for EnumError {
    fn from(_error: std::io::Error) -> Self {
        EnumError::Io(())
    }
}

impl From<zip::result::ZipError> for EnumError {
    fn from(_error: zip::result::ZipError) -> Self {
        EnumError::Zip(())
    }
}

#[derive(Debug, Serialize)]
struct ComputerAccount {
    distinguished_name: String,
    cn: String,
    dns_hostname: Option<String>,
    operating_system: Option<String>,
    os_version: Option<String>,
    when_created: Option<String>,
    last_logon: Option<String>,
}

#[derive(Debug, Serialize)]
struct UserAccount {
    distinguished_name: String,
    cn: String,
    sam_account_name: Option<String>,
    sid: Option<String>,
    when_created: Option<String>,
    last_logon: Option<String>,
}

#[derive(Debug, Serialize)]
struct Group {
    distinguished_name: String,
    cn: String,
    sid: Option<String>,
    members: Vec<String>,
}

// Configuration struct for Tauri commands
#[derive(Debug, Deserialize)]
pub struct LdapIndexConfig {
    pub dc_hostname: String,
    pub base_dn: String,
    pub username: String,
    pub password: String,
    pub use_ldaps: bool,
    pub mode: String,
    pub debug_mode: bool,
}

// Progress update structure for frontend
#[derive(Debug, Serialize, Clone)]
pub struct ProgressUpdate {
    pub message: String,
    pub current: usize,
    pub total: Option<usize>,
    pub stage: String, // "connecting", "enumerating", "complete", "error"
}

// Final result structure
#[derive(Debug, Serialize)]
pub struct IndexResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
    pub total_entries: usize,
    pub errors: Vec<String>,
}

fn filetime_to_datetime(filetime: &str) -> Option<String> {
    filetime.parse::<i64>().ok().and_then(|ft| {
        if ft == 0 {
            None
        } else {
            let seconds_since_windows_epoch = ft / 10_000_000;
            let unix_epoch = seconds_since_windows_epoch - 11644473600;
            let datetime = DateTime::<Utc>::from_timestamp(unix_epoch, 0)?;
            Some(datetime.to_rfc3339())
        }
    })
}

// Parse a Windows SID from binary form to string representation (e.g., "S-1-5-21-...")
fn parse_sid(sid_bytes: &[u8]) -> Option<String> {
    if sid_bytes.len() < 8 {
        return None; // SID too short
    }

    let revision = sid_bytes[0];
    let sub_authority_count = sid_bytes[1] as usize;
    
    if sid_bytes.len() < 8 + (sub_authority_count * 4) {
        return None; // SID data incomplete
    }
    
    // Authority is a 48-bit value stored in big-endian
    let authority = ((sid_bytes[2] as u64) << 40) |
                   ((sid_bytes[3] as u64) << 32) |
                   ((sid_bytes[4] as u64) << 24) |
                   ((sid_bytes[5] as u64) << 16) |
                   ((sid_bytes[6] as u64) << 8) |
                    (sid_bytes[7] as u64);
    
    let mut result = format!("S-{}-{}", revision, authority);
    
    // Sub-authorities are stored in little-endian
    for i in 0..sub_authority_count {
        let offset = 8 + (i * 4);
        let sub_authority = 
            ((sid_bytes[offset] as u32)) |
            ((sid_bytes[offset + 1] as u32) << 8) |
            ((sid_bytes[offset + 2] as u32) << 16) |
            ((sid_bytes[offset + 3] as u32) << 24);
        
        result.push_str(&format!("-{}", sub_authority));
    }
    
    Some(result)
}

/// Send progress update to frontend
fn send_progress_update(
    window: &Window,
    message: String,
    current: usize,
    total: Option<usize>,
    stage: String,
) {
    let update = ProgressUpdate {
        message,
        current,
        total,
        stage,
    };
    
    let _ = window.emit("indexing-progress", &update);
}

/// Send log message to frontend
fn send_log_message(window: &Window, message: String) {
    let _ = window.emit("indexing-log", message);
}

async fn ldap_search(
    ldap: &mut ldap3::Ldap,
    base_dn: &str,
    filter: &str,
    attrs: Vec<&str>,
    window: Option<&Window>,
    debug_mode: bool,
) -> ldap3::result::Result<Vec<SearchEntry>> {
    if let Some(win) = window {
        if debug_mode {
            send_log_message(win, format!("Searching with filter: {}", filter));
        }
    }
    
    // Use paged search with adapter chain
    let adapters: Vec<Box<dyn Adapter<'_, &str, Vec<&str>>>> = vec![
        Box::new(PagedResults::new(1000)), // AD default MaxPageSize is 1000
        Box::new(EntriesOnly::new()),
    ];

    let mut stream = ldap
        .streaming_search_with(
            adapters,
            base_dn,
            Scope::Subtree,
            filter,
            attrs,
        )
        .await?;

    let mut entries = Vec::new();
    
    // Iterate through all pages and entries
    while let Some(re) = stream.next().await? {
        // Safe: EntriesOnly ensures `re` is a real SearchResultEntry.
        let se = SearchEntry::construct(re);
        entries.push(se);
    }

    // Ensure the overall search completed successfully
    stream.finish().await.success()?;
    
    if let Some(win) = window {
        if debug_mode {
            send_log_message(win, format!("Retrieved {} entries", entries.len()));
        }
    }
    
    Ok(entries)
}

async fn enumerate_computers(
    ldap: &mut ldap3::Ldap, 
    base_dn: &str,
    output_path: &str,
    window: &Window,
    debug_mode: bool
) -> Result<usize, EnumError> {
    send_log_message(window, "Enumerating computer accounts...".to_string());
    
    let entries = ldap_search(
        ldap, 
        base_dn, 
        "(objectClass=computer)", 
        vec![
            "distinguishedName", "cn", "dNSHostName", "operatingSystem",
            "operatingSystemVersion", "whenCreated", "lastLogonTimestamp"
        ],
        Some(window),
        debug_mode
    ).await?;

    send_progress_update(
        window,
        format!("Found {} computer accounts", entries.len()),
        0,
        Some(entries.len()),
        "enumerating".to_string(),
    );

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for (i, entry) in entries.iter().enumerate() {
        if i % 100 == 0 {
            send_progress_update(
                window,
                format!("Processing computer accounts"),
                i,
                Some(entries.len()),
                "enumerating".to_string(),
            );
        }
        
        let computer = ComputerAccount {
            distinguished_name: entry.attrs.get("distinguishedName").and_then(|v| v.get(0)).cloned().unwrap_or_default(),
            cn: entry.attrs.get("cn").and_then(|v| v.get(0)).cloned().unwrap_or_default(),
            dns_hostname: entry.attrs.get("dNSHostName").and_then(|v| v.get(0)).cloned(),
            operating_system: entry.attrs.get("operatingSystem").and_then(|v| v.get(0)).cloned(),
            os_version: entry.attrs.get("operatingSystemVersion").and_then(|v| v.get(0)).cloned(),
            when_created: entry.attrs.get("whenCreated").and_then(|v| v.get(0)).cloned(),
            last_logon: entry.attrs.get("lastLogonTimestamp").and_then(|v| v.get(0)).and_then(|val| filetime_to_datetime(val)),
        };
        to_writer(&mut writer, &computer)?;
        writer.write_all(b"\n")?;
    }
    
    send_log_message(window, format!("Completed computer enumeration: {} accounts", entries.len()));
    Ok(entries.len())
}

fn get_string_attr(entry: &SearchEntry, attr_name: &str) -> Option<String> {
    entry.attrs.get(attr_name).and_then(|v| v.get(0)).cloned()
}

// Returns well-known SIDs for built-in groups
// Attention this only works when OS is in English - best would be rework of enumeration so we get the SID. - Curerntly this problem is solved on the import as we create all builtin groups and add members to them.
fn get_well_known_sid(group_name: &str) -> Option<String> {
    match group_name {
        "Administrators" => Some("S-1-5-32-544".to_string()),
        "Users" => Some("S-1-5-32-545".to_string()),
        "Guests" => Some("S-1-5-32-546".to_string()),
        "Power Users" => Some("S-1-5-32-547".to_string()),
        "Account Operators" => Some("S-1-5-32-548".to_string()),
        "Server Operators" => Some("S-1-5-32-549".to_string()),
        "Print Operators" => Some("S-1-5-32-550".to_string()),
        "Backup Operators" => Some("S-1-5-32-551".to_string()),
        "Replicators" => Some("S-1-5-32-552".to_string()),
        "Replicator" => Some("S-1-5-32-552".to_string()), // For backwards compatibility
        "Pre-Windows 2000 Compatible Access" => Some("S-1-5-32-554".to_string()),
        "Remote Desktop Users" => Some("S-1-5-32-555".to_string()),
        "Network Configuration Operators" => Some("S-1-5-32-556".to_string()),
        "Incoming Forest Trust Builders" => Some("S-1-5-32-557".to_string()),
        "Performance Monitor Users" => Some("S-1-5-32-558".to_string()),
        "Performance Log Users" => Some("S-1-5-32-559".to_string()),
        "Windows Authorization Access Group" => Some("S-1-5-32-560".to_string()),
        "Terminal Server License Servers" => Some("S-1-5-32-561".to_string()),
        "Distributed COM Users" => Some("S-1-5-32-562".to_string()),
        "IIS_IUSRS" => Some("S-1-5-32-568".to_string()),
        "Cryptographic Operators" => Some("S-1-5-32-569".to_string()),
        "Event Log Readers" => Some("S-1-5-32-573".to_string()),
        "Certificate Service DCOM Access" => Some("S-1-5-32-574".to_string()),
        "RDS Remote Access Servers" => Some("S-1-5-32-575".to_string()),
        "RDS Endpoint Servers" => Some("S-1-5-32-576".to_string()),
        "RDS Management Servers" => Some("S-1-5-32-577".to_string()),
        "Hyper-V Administrators" => Some("S-1-5-32-578".to_string()),
        "Access Control Assistance Operators" => Some("S-1-5-32-579".to_string()),
        "Remote Management Users" => Some("S-1-5-32-580".to_string()),
        _ => None,
    }
}

fn get_sid_attr(entry: &SearchEntry, attr_name: &str) -> Option<String> {
    // Try to get SID from binary attribute
    if let Some(vals) = entry.bin_attrs.get(attr_name) {
        if let Some(bytes) = vals.get(0) {
            return parse_sid(bytes);
        }
    }
    
    // If it's a built-in group, fall back to well-known SIDs
    if entry.dn.contains("CN=Builtin,") {
        let cn = get_string_attr(entry, "cn").unwrap_or_default();
        return get_well_known_sid(&cn);
    }
    
    None
}

async fn enumerate_users(
    ldap: &mut ldap3::Ldap, 
    base_dn: &str,
    output_path: &str,
    window: &Window,
    debug_mode: bool
) -> Result<usize, EnumError> {
    send_log_message(window, "Enumerating user accounts...".to_string());
    
    let attrs_to_request = vec![
        "distinguishedName", "cn", "sAMAccountName", "objectSid",
        "whenCreated", "lastLogonTimestamp"
    ];
    
    // Use a specific filter that gets users but excludes computer accounts
    let filter = "(&(objectClass=user)(objectCategory=person))";
    
    let entries = ldap_search(
        ldap, 
        base_dn, 
        filter, 
        attrs_to_request,
        Some(window),
        debug_mode
    ).await?;

    send_progress_update(
        window,
        format!("Found {} user accounts", entries.len()),
        0,
        Some(entries.len()),
        "enumerating".to_string(),
    );

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for (i, entry) in entries.iter().enumerate() {
        if i % 100 == 0 {
            send_progress_update(
                window,
                format!("Processing user accounts"),
                i,
                Some(entries.len()),
                "enumerating".to_string(),
            );
        }
        
        let user = UserAccount {
            distinguished_name: get_string_attr(entry, "distinguishedName").unwrap_or_default(),
            cn: get_string_attr(entry, "cn").unwrap_or_default(),
            sam_account_name: get_string_attr(entry, "sAMAccountName"),
            sid: get_sid_attr(entry, "objectSid"),
            when_created: get_string_attr(entry, "whenCreated"),
            last_logon: entry.attrs.get("lastLogonTimestamp")
                         .and_then(|v| v.get(0))
                         .and_then(|val| filetime_to_datetime(val)),
        };
        to_writer(&mut writer, &user)?;
        writer.write_all(b"\n")?;
    }
    
    send_log_message(window, format!("Completed user enumeration: {} accounts", entries.len()));
    Ok(entries.len())
}

async fn enumerate_groups(
    ldap: &mut ldap3::Ldap, 
    base_dn: &str,
    output_path: &str,
    window: &Window,
    debug_mode: bool
) -> Result<usize, EnumError> {
    send_log_message(window, "Enumerating groups...".to_string());
    
    let entries = ldap_search(
        ldap, 
        base_dn, 
        "(objectClass=group)", 
        vec!["distinguishedName", "cn", "member", "objectSid"],
        Some(window),
        debug_mode
    ).await?;

    send_progress_update(
        window,
        format!("Found {} groups", entries.len()),
        0,
        Some(entries.len()),
        "enumerating".to_string(),
    );

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for (i, entry) in entries.iter().enumerate() {
        if i % 100 == 0 {
            send_progress_update(
                window,
                format!("Processing groups"),
                i,
                Some(entries.len()),
                "enumerating".to_string(),
            );
        }
        
        let cn = get_string_attr(entry, "cn").unwrap_or_default();
        let sid = get_sid_attr(entry, "objectSid").or_else(|| Some(cn.clone()));

        let group = Group {
            distinguished_name: get_string_attr(entry, "distinguishedName").unwrap_or_default(),
            cn,
            sid,
            members: entry.attrs.get("member").cloned().unwrap_or_default(),
        };
        to_writer(&mut writer, &group)?;
        writer.write_all(b"\n")?;
    }
    
    send_log_message(window, format!("Completed group enumeration: {} groups", entries.len()));
    Ok(entries.len())
}

fn handle_enumeration_error(error: EnumError, mode: &str, _base_dn: &str, window: &Window) -> String {
    match &error {
        EnumError::Ldap(ldap_error) => {
            let error_str = ldap_error.to_string();
            if error_str.contains("rc:4") || error_str.contains("Size limit exceeded") {
                let message = format!("ERROR: Size limit exceeded while enumerating {}. \
                                      This means the server has a hard limit that prevents returning results. \
                                      Try a more specific base DN or contact your AD administrator.", mode);
                send_log_message(window, message.clone());
                message
            } else {
                let message = format!("LDAP Error during {} enumeration: {}", mode, ldap_error);
                send_log_message(window, message.clone());
                message
            }
        }
        other => {
            let message = format!("Error during {} enumeration: {:?}", mode, other);
            send_log_message(window, message.clone());
            message
        }
    }
}

/// Create a zip archive containing all output files
fn create_zip_archive(files: &[String], window: &Window) -> Result<String, EnumError> {
    if files.is_empty() {
        // Create error without keeping the std::io::Error details
        let _ = std::io::Error::new(std::io::ErrorKind::InvalidInput, "No files to archive");
        return Err(EnumError::Io(()));
    }

    // Create a zip file with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let zip_path = format!("domain_data_{}.zip", timestamp);
    
    send_log_message(window, format!("Creating zip archive: {}", zip_path));
    
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for path in files {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        
        // Extract just the filename without the path
        let filename = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        send_log_message(window, format!("Adding {} to archive", filename));
        zip.start_file(filename, options)?;
        std::io::copy(&mut reader, &mut zip)?;
    }

    zip.finish()?;
    send_log_message(window, format!("Zip archive created: {}", zip_path));
    
    Ok(zip_path)
}

// Function to derive base DN from UPN format username
fn derive_base_dn_from_upn(username: &str) -> Option<String> {
    if let Some(domain_part) = username.split('@').nth(1) {
        let components: Vec<&str> = domain_part.split('.').collect();
        if components.len() >= 2 {
            let dn = components.iter()
                .map(|component| format!("DC={}", component))
                .collect::<Vec<String>>()
                .join(",");
            Some(dn)
        } else {
            None
        }
    } else {
        None
    }
}

// Function to query LDAP root DSE for default naming context
async fn get_default_naming_context(ldap: &mut ldap3::Ldap, window: &Window, debug_mode: bool) -> Result<Option<String>, EnumError> {
    send_log_message(window, "Querying LDAP server for default naming context...".to_string());
    
    match ldap_search(ldap, "", "(objectClass=*)", vec!["defaultNamingContext"], Some(window), debug_mode).await {
        Ok(entries) => {
            if let Some(entry) = entries.first() {
                if let Some(default_nc) = entry.attrs.get("defaultNamingContext") {
                    if let Some(dn) = default_nc.first() {
                        send_log_message(window, format!("Found default naming context: {}", dn));
                        return Ok(Some(dn.clone()));
                    }
                }
            }
            send_log_message(window, "No default naming context found in root DSE".to_string());
            Ok(None)
        }
        Err(e) => {
            send_log_message(window, format!("Failed to query root DSE: {}", e));
            Ok(None)
        }
    }
}

/// Main Tauri command for LDAP enumeration
#[tauri::command]
pub async fn start_ldap_enumeration(
    window: Window,
    config: LdapIndexConfig,
) -> Result<IndexResult, String> {
    // Send initial progress
    send_progress_update(
        &window,
        "Starting LDAP enumeration...".to_string(),
        0,
        None,
        "connecting".to_string(),
    );

    send_log_message(&window, "LDAP enumeration started".to_string());
    
    // Create output files with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let computers_file = format!("computers_{}.jsonl", timestamp);
    let users_file = format!("users_{}.jsonl", timestamp);
    let groups_file = format!("groups_{}.jsonl", timestamp);
    
    // Initialize output files list
    let mut output_files = Vec::new();
    let mut total_entries = 0;
    let mut errors = Vec::new();
    
    // Adjust LDAPS port if not specified
    let dc_hostname = config.dc_hostname.trim();
    let ldap_url = if config.use_ldaps && !dc_hostname.contains(':') {
        format!("ldaps://{}:636", dc_hostname)
    } else if !config.use_ldaps && !dc_hostname.contains(':') {
        format!("ldap://{}:389", dc_hostname)
    } else {
        let protocol = if config.use_ldaps { "ldaps" } else { "ldap" };
        format!("{}://{}", protocol, dc_hostname)
    };

    send_log_message(&window, format!("Connecting to: {}", ldap_url));
    if config.use_ldaps {
        send_log_message(&window, "Using LDAPS (TLS encrypted connection)".to_string());
    } else {
        send_log_message(&window, "Using plain LDAP (unencrypted connection)".to_string());
    }

    // Build connection settings with TLS certificate validation disabled
    let settings = LdapConnSettings::new()
        .set_no_tls_verify(true);

    // Connect to LDAP server
    let (conn, mut ldap) = match LdapConnAsync::with_settings(settings, &ldap_url).await {
        Ok((conn, ldap)) => (conn, ldap),
        Err(e) => {
            let error_msg = format!("Failed to connect to LDAP server: {}", e);
            send_log_message(&window, error_msg.clone());
            send_progress_update(
                &window,
                error_msg.clone(),
                0,
                None,
                "error".to_string(),
            );
            return Ok(IndexResult {
                success: false,
                message: error_msg,
                output_files,
                total_entries: 0,
                errors: vec![e.to_string()],
            });
        }
    };
    
    ldap3::drive!(conn);
    
    // Authenticate
    let username = config.username.trim();
    let password = config.password.trim();
    
    if !username.is_empty() && !password.is_empty() {
        send_log_message(&window, "Binding with provided credentials...".to_string());
        match ldap.simple_bind(username, password).await {
            Ok(result) => match result.success() {
                Ok(_) => send_log_message(&window, "Authentication successful".to_string()),
                Err(e) => {
                    let error_msg = format!("Authentication failed: {}", e);
                    send_log_message(&window, error_msg.clone());
                    send_progress_update(
                        &window,
                        error_msg.clone(),
                        0,
                        None,
                        "error".to_string(),
                    );
                    return Ok(IndexResult {
                        success: false,
                        message: error_msg,
                        output_files,
                        total_entries: 0,
                        errors: vec![e.to_string()],
                    });
                }
            },
            Err(e) => {
                let mut error_msg = format!("Bind failed: {}", e);
                
                if e.to_string().contains("integrity checking") || e.to_string().contains("signing") {
                    error_msg = "LDAP signing is required by the server. Try using LDAPS instead.".to_string();
                }
                
                send_log_message(&window, error_msg.clone());
                send_progress_update(
                    &window,
                    error_msg.clone(),
                    0,
                    None,
                    "error".to_string(),
                );
                return Ok(IndexResult {
                    success: false,
                    message: error_msg,
                    output_files,
                    total_entries: 0,
                    errors: vec![e.to_string()],
                });
            }
        }
    } else {
        // Try Kerberos auth via GSSAPI only
        send_log_message(&window, "No credentials provided, attempting Kerberos authentication via GSSAPI...".to_string());
        match ldap.sasl_gssapi_bind("").await {
            Ok(result) => match result.success() {
                Ok(_) => send_log_message(&window, "Kerberos GSSAPI authentication successful".to_string()),
                Err(e) => {
                    let error_msg = format!("Kerberos GSSAPI authentication failed: {}", e);
                    send_log_message(&window, error_msg.clone());
                    send_progress_update(
                        &window,
                        error_msg.clone(),
                        0,
                        None,
                        "error".to_string(),
                    );
                    return Ok(IndexResult {
                        success: false,
                        message: error_msg,
                        output_files,
                        total_entries: 0,
                        errors: vec![e.to_string()],
                    });
                }
            },
            Err(e) => {
                let error_msg = format!("Kerberos GSSAPI bind failed: {}", e);
                send_log_message(&window, error_msg.clone());
                send_progress_update(
                    &window,
                    error_msg.clone(),
                    0,
                    None,
                    "error".to_string(),
                );
                return Ok(IndexResult {
                    success: false,
                    message: error_msg,
                    output_files,
                    total_entries: 0,
                    errors: vec![e.to_string()],
                });
            }
        }
    }

    // Get base DN - use provided value, derive from username, or query server
    let mut base_dn = config.base_dn.trim().to_string();
    
    if base_dn.is_empty() {
        send_log_message(&window, "Base DN not provided. Attempting auto-detection...".to_string());
        
        // First try to derive from UPN username
        if !username.is_empty() && username.contains('@') {
            if let Some(upn_base_dn) = derive_base_dn_from_upn(username) {
                base_dn = upn_base_dn;
                send_log_message(&window, format!("Derived base DN from username: {}", base_dn));
            }
        }
        
        // If still empty, query LDAP server for default naming context
        if base_dn.is_empty() {
            match get_default_naming_context(&mut ldap, &window, config.debug_mode).await {
                Ok(Some(default_naming_context)) => {
                    base_dn = default_naming_context;
                    send_log_message(&window, format!("Found default naming context: {}", base_dn));
                }
                Ok(None) => {
                    let error_msg = "Could not auto-detect base DN. Please specify a base DN.".to_string();
                    send_log_message(&window, error_msg.clone());
                    send_progress_update(
                        &window,
                        error_msg.clone(),
                        0,
                        None,
                        "error".to_string(),
                    );
                    return Ok(IndexResult {
                        success: false,
                        message: error_msg.clone(),
                        output_files,
                        total_entries: 0,
                        errors: vec![error_msg],
                    });
                }
                Err(e) => {
                    let error_msg = format!("Error querying for default naming context: {:?}", e);
                    send_log_message(&window, error_msg.clone());
                    send_progress_update(
                        &window,
                        error_msg.clone(),
                        0,
                        None,
                        "error".to_string(),
                    );
                    return Ok(IndexResult {
                        success: false,
                        message: error_msg,
                        output_files,
                        total_entries: 0,
                        errors: vec![format!("{:?}", e)],
                    });
                }
            }
        }
    } else {
        send_log_message(&window, format!("Using specified base DN: {}", base_dn));
    }

    // Execute enumeration based on mode
    let mode = config.mode.to_lowercase();
    let mut overall_success = true;

    if mode == "computers" || mode == "all" {
        match enumerate_computers(&mut ldap, &base_dn, &computers_file, &window, config.debug_mode).await {
            Ok(count) => {
                output_files.push(computers_file.clone());
                total_entries += count;
            },
            Err(e) => {
                overall_success = false;
                let error_message = handle_enumeration_error(e, "computers", &base_dn, &window);
                errors.push(error_message);
            }
        }
    }
    
    if mode == "users" || mode == "all" {
        match enumerate_users(&mut ldap, &base_dn, &users_file, &window, config.debug_mode).await {
            Ok(count) => {
                output_files.push(users_file.clone());
                total_entries += count;
            },
            Err(e) => {
                overall_success = false;
                let error_message = handle_enumeration_error(e, "users", &base_dn, &window);
                errors.push(error_message);
            }
        }
    }
    
    if mode == "groups" || mode == "all" {
        match enumerate_groups(&mut ldap, &base_dn, &groups_file, &window, config.debug_mode).await {
            Ok(count) => {
                output_files.push(groups_file.clone());
                total_entries += count;
            },
            Err(e) => {
                overall_success = false;
                let error_message = handle_enumeration_error(e, "groups", &base_dn, &window);
                errors.push(error_message);
            }
        }
    }

    // Close the connection
    if let Err(e) = ldap.unbind().await {
        send_log_message(&window, format!("Error during LDAP unbind: {}", e));
    }

    // Send final progress update
    let stage = if overall_success { "complete" } else { "error" };
    let message = if overall_success { 
        "LDAP enumeration completed successfully" 
    } else { 
        "LDAP enumeration completed with errors" 
    };
    
    send_progress_update(
        &window,
        message.to_string(),
        total_entries,
        None,
        stage.to_string(),
    );
    
    send_log_message(&window, format!("Enumeration complete. Total entries: {}", total_entries));
    
    // Create zip archive if there are output files
    if !output_files.is_empty() {
        send_log_message(&window, "Creating zip archive of all output files...".to_string());
        
        match create_zip_archive(&output_files, &window) {
            Ok(zip_path) => {
                // Delete original files
                for file in &output_files {
                    if let Err(e) = remove_file(file) {
                        send_log_message(&window, format!("Warning: Failed to delete file {}: {}", file, e));
                    } else {
                        send_log_message(&window, format!("Deleted original file: {}", file));
                    }
                }
                
                // Replace output_files with just the zip file
                output_files = vec![zip_path];
            },
            Err(e) => {
                let error_message = format!("Failed to create zip archive: {:?}", e);
                send_log_message(&window, error_message.clone());
                errors.push(error_message);
            }
        }
    }
    
    // Return the result
    Ok(IndexResult {
        success: overall_success,
        message: message.to_string(),
        output_files,
        total_entries,
        errors,
    })
}
