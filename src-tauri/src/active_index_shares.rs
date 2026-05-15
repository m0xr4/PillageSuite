// Modules and imports for better organization
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    ptr::null_mut,
    slice,
    sync::mpsc::{self, Receiver, Sender},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};

use chrono;
use serde_json;
use serde::{Serialize, Deserialize};
use tauri::{Window, Emitter};
use windows::{
    core::{PCWSTR, Result as WinResult},
    Win32::{
        Storage::FileSystem::{
            NetShareEnum, SHARE_INFO_0
        },
        NetworkManagement::NetManagement::{
            NetApiBufferFree
        },
        Security::{
            ACCESS_ALLOWED_ACE, ACCESS_DENIED_ACE, ACE_HEADER, DACL_SECURITY_INFORMATION,
            GetAce, GetFileSecurityW, GetSecurityDescriptorDacl, GROUP_SECURITY_INFORMATION,
            OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        },
    },
};

// External Windows API definitions
#[link(name = "kernel32")]
extern "system" {
    fn LocalFree(hMem: isize) -> isize;
}

#[link(name = "advapi32")]
extern "system" {
    fn ConvertSidToStringSidW(Sid: PSID, StringSid: *mut *mut u16) -> i32;
}

/// Global abort flag checked by all workers and writer
static INDEX_ABORT_REQUESTED: AtomicBool = AtomicBool::new(false);

//========================================================================
// THREADING TYPES
//========================================================================

/// Messages sent from worker threads to the writer thread
enum WorkerOutput {
    JsonlRecord(String),
    TextLine(String),
    Log(String),
    Error(String),
    Done,
}

/// Configuration shared across worker threads
struct WorkerConfig {
    max_depth: usize,
    max_entries: Option<usize>,
    debug_mode: bool,
    smb_username: String,
    smb_password: String,
    smb_domain: String,
}

/// Work-stealing queue: workers pull next item via AtomicUsize index
struct WorkQueue {
    items: Vec<String>,
    next_index: AtomicUsize,
}

impl WorkQueue {
    fn new(items: Vec<String>) -> Self {
        WorkQueue {
            items,
            next_index: AtomicUsize::new(0),
        }
    }

    fn next(&self) -> Option<String> {
        let idx = self.next_index.fetch_add(1, Ordering::Relaxed);
        self.items.get(idx).cloned()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

//========================================================================
// CONFIG AND TAURI STRUCTURES
//========================================================================

/// Configuration struct for Tauri commands
#[derive(Debug, Deserialize)]
pub struct IndexConfig {
    pub targets: String,  // Comma-separated hosts or file path
    pub max_depth: usize,
    pub max_entries: Option<usize>,
    pub debug_mode: bool,
    pub share_enum_only: bool,
    pub shares_file: Option<String>,
    pub smb_username: Option<String>,
    pub smb_password: Option<String>,
    pub smb_domain: Option<String>,
    pub thread_count: Option<usize>,
}

/// Progress update structure for frontend
#[derive(Debug, Serialize, Clone)]
pub struct ProgressUpdate {
    pub message: String,
    pub current: usize,
    pub total: Option<usize>,
    pub stage: String, // "connecting", "enumerating", "walking", "complete"
}

/// Final result structure
#[derive(Debug, Serialize)]
pub struct IndexResult {
    pub success: bool,
    pub message: String,
    pub output_file: String,
    pub total_entries: usize,
    pub errors: Vec<String>,
}

/// Internal configuration struct
#[derive(Debug)]
struct Config {
    target_or_file: String,
    max_depth: usize,
    output_path: String,
    max_entries: Option<usize>,
    debug_mode: bool,
    share_enum_only: bool,
    shares_file: Option<String>,
}

impl Config {
    /// Create Config from IndexConfig (from frontend)
    fn from_index_config(index_config: IndexConfig, output_path: String) -> Self {
        Config {
            target_or_file: index_config.targets,
            max_depth: index_config.max_depth,
            output_path,
            max_entries: index_config.max_entries,
            debug_mode: index_config.debug_mode,
            share_enum_only: index_config.share_enum_only,
            shares_file: index_config.shares_file,
        }
    }
}

//========================================================================
// DATA MODELS
//========================================================================

/// File metadata to be serialized line-by-line (NDJSON).
#[derive(Debug, Serialize)]
struct FileMetadata {
    name: String,
    full_path: String,
    size: Option<u64>,
    extension: Option<String>,
    created: Option<String>,
    modified: Option<String>,
    acls: Option<Vec<AceInfo>>,
    entry_type: String,  // "file", "directory", or "share"
}

/// Info on each ACE in the DACL.
#[derive(Debug, Serialize)]
struct AceInfo {
    identity: String,         // SID string (S-1-5-...)
    ace_type: String,         // "ALLOWED" or "DENIED"
    access_mask: u32,         // raw mask
    permissions: Vec<String>, // e.g. ["FullControl", "Modify", "GenericRead"]
}

//========================================================================
// SHARE ENUMERATION FUNCTIONS
//========================================================================

/// Enumerate shares on a given server/host (e.g. "MYHOST"), returning share names
/// like ["C$", "Public", "IPC$", etc.].
fn enumerate_shares(host_name: &str) -> WinResult<Vec<String>> {
    let mut buf_ptr: *mut u8 = null_mut();
    let mut entries_read: u32 = 0;
    let mut total_entries: u32 = 0;

    let host_wide = string_to_wide(host_name);

    let status = unsafe {
        NetShareEnum(
            PCWSTR(host_wide.as_ptr()),
            0,  // Use level 0 for SHARE_INFO_0 (just share names)
            &mut buf_ptr,
            32768,
            &mut entries_read,
            &mut total_entries,
            None,
        )
    };

    if status != 0 {
        return Err(windows::core::Error::from_win32());
    }

    let mut shares = Vec::new();
    if !buf_ptr.is_null() && entries_read > 0 {
        let share_array =
            unsafe { slice::from_raw_parts(buf_ptr as *const SHARE_INFO_0, entries_read as usize) };
        for share_info in share_array {
            let share_name = wide_str_to_string(share_info.shi0_netname.0 as *const u16);
            if !share_name.is_empty() {
                shares.push(share_name);
            }
        }
    }

    unsafe {
        if !buf_ptr.is_null() {
            NetApiBufferFree(Some(buf_ptr as *const std::ffi::c_void));
        }
    }

    Ok(shares)
}

/// Load hosts from a file or return a single host if the input is not a file path
fn load_hosts(target_or_file: &str) -> Vec<String> {
    if std::path::Path::new(target_or_file).exists() {
        // It's a file: read hostnames line by line
        match std::fs::File::open(target_or_file) {
            Ok(f) => BufReader::new(f)
                .lines()
                .filter_map(|l| l.ok())
                .filter(|line| !line.trim().is_empty())
                .collect(),
            Err(_) => {
                // Return empty vec on error - error will be handled at higher level
                Vec::new()
            }
        }
    } else {
        // It's a single hostname
        vec![target_or_file.to_string()]
    }
}

/// Load UNC paths from a file
fn load_shares_from_file(shares_file: &str) -> Vec<String> {
    match std::fs::File::open(shares_file) {
        Ok(f) => BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .filter(|line| !line.trim().is_empty())
            .collect(),
        Err(_) => {
            // Return empty vec on error - error will be handled at higher level
            Vec::new()
        }
    }
}

/// Create a BufWriter for output file
fn create_output_writer(output_path: &str) -> Result<BufWriter<std::fs::File>, std::io::Error> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)?;
    
    Ok(BufWriter::new(file))
}

/// Should a share be skipped (common admin shares)
fn should_skip_share(share: &str) -> bool {
    let lower = share.to_lowercase();
    lower == "admin$" || lower == "ipc$" || lower == "print$"
}

//========================================================================
// PROGRESS REPORTING FUNCTIONS
//========================================================================

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

//========================================================================
// FILE/SHARE WALKING FUNCTIONS
//========================================================================

/// Recursively walk a UNC path up to `max_depth`. For each file/folder found,
/// retrieve metadata/ACLs and write them to NDJSON (one record per line).
fn walk_share_unc(
    window: &Window,
    unc_path: &str,
    current_depth: usize,
    max_depth: usize,
    max_entries: Option<usize>,
    writer: &mut BufWriter<std::fs::File>,
    debug_mode: bool,
    global_count: &mut usize,
) -> usize {
    // Keep track of total entries processed
    let mut entries_count = 0;

    // Send progress update if this is the root level
    if current_depth == 0 {
        send_progress_update(
            window,
            format!("Walking share: {}", unc_path),
            *global_count,
            max_entries,
            "walking".to_string(),
        );
        
        // First, create an entry for the share root
        let added = process_share_root(unc_path, writer, debug_mode);
        entries_count += added;
        *global_count += added;
        
        // Update progress after processing the share root
        send_progress_update(
            window,
            format!("Processing share: {}", unc_path),
            *global_count,
            max_entries,
            "walking".to_string(),
        );
    }

    if current_depth > max_depth {
        if current_depth == 0 {
            send_progress_update(
                window,
                "Reached maximum depth".to_string(),
                entries_count,
                max_entries,
                "complete".to_string(),
            );
        }
        return entries_count;
    }

    // Check if we've reached the max entries limit
    if let Some(limit) = max_entries {
        if *global_count >= limit {
            if debug_mode {
                send_log_message(window, format!("Reached max entries limit ({}) for share: {}", limit, unc_path));
            }
            if current_depth == 0 {
                send_progress_update(
                    window,
                    "Reached maximum entries limit".to_string(),
                    *global_count,
                    max_entries,
                    "complete".to_string(),
                );
            }
            return entries_count;
        }
    }

    let path = PathBuf::from(unc_path);
    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(_) => {
            // Permission denied or not a directory, skip
            if current_depth == 0 {
                send_progress_update(
                    window,
                    "Access denied or invalid path".to_string(),
                    *global_count,
                    max_entries,
                    "complete".to_string(),
                );
            }
            return entries_count;
        }
    };

    for entry_result in entries {
        // Check if we've reached the max entries limit
        if let Some(limit) = max_entries {
            if *global_count >= limit {
                if debug_mode {
                    send_log_message(window, format!("Reached max entries limit ({}) for share: {}", limit, unc_path));
                }
                if current_depth == 0 {
                    send_progress_update(
                        window,
                        "Reached maximum entries limit".to_string(),
                        *global_count,
                        max_entries,
                        "complete".to_string(),
                    );
                }
                return entries_count;
            }
        }
        
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = entry.path();

        // Process this entry
        let added = process_filesystem_entry(&entry, writer, debug_mode);
        entries_count += added;
        *global_count += added;
        
        // Update progress at any depth to avoid UI stall during deep recursion
        send_progress_update(
            window,
            format!("Processed {} entries", *global_count),
            *global_count,
            max_entries,
            "walking".to_string(),
        );

        // Recurse if directory
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            let full_path = entry_path.to_string_lossy().to_string();
            let sub_entries = walk_share_unc(window, &full_path, current_depth + 1, max_depth, max_entries, writer, debug_mode, global_count);
            entries_count += sub_entries;
            
            // Update progress after recursion
            send_progress_update(
                window,
                format!("Processed {} entries", *global_count),
                *global_count,
                max_entries,
                "walking".to_string(),
            );
            
            // Check again after recursion
            if let Some(limit) = max_entries {
                if *global_count >= limit {
                    if debug_mode {
                        send_log_message(window, format!("Reached max entries limit ({}) after recursion for: {}", limit, full_path));
                    }
                    if current_depth == 0 {
                        send_progress_update(
                            window,
                            "Reached maximum entries limit".to_string(),
                            *global_count,
                            max_entries,
                            "complete".to_string(),
                        );
                    }
                    return entries_count;
                }
            }
        }
    }
    
    // Send completion update if we're at the root level
    if current_depth == 0 {
        send_progress_update(
            window,
            format!("Completed walking share: {} entries processed", *global_count),
            *global_count,
            max_entries,
            "complete".to_string(),
        );
    }
    
    entries_count
}

/// Process a share root and create an entry for it
fn process_share_root(unc_path: &str, writer: &mut BufWriter<std::fs::File>, debug_mode: bool) -> usize {
    // Get share metadata
    let path = PathBuf::from(unc_path);
    match fs::metadata(&path) {
        Ok(metadata) => {
            let created_str = metadata.created().ok().and_then(system_time_to_string);
            let modified_str = metadata.modified().ok().and_then(system_time_to_string);
            
            // Attempt to fetch ACL info
            let acls = match get_acl_info(&path, debug_mode, None) {
                Ok(a) => Some(a),
                Err(_) => None,
            };

            // Create share root entry
            let share_meta = FileMetadata {
                name: unc_path.to_string(),
                full_path: unc_path.to_string(),
                size: None, // Shares don't have a meaningful size
                extension: None,
                created: created_str,
                modified: modified_str,
                acls,
                entry_type: "share".to_string(),
            };

            if let Err(_e) = write_json_line(&share_meta, writer) {
                // Error handling - could be logged if needed
            }
            
            1 // Return 1 for the entry created
        },
        Err(_) => 0 // Return 0 if we couldn't get metadata
    }
}

/// Process a filesystem entry (file or directory) and create an entry for it
fn process_filesystem_entry(
    entry: &fs::DirEntry, 
    writer: &mut BufWriter<std::fs::File>, 
    debug_mode: bool
) -> usize {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let metadata = match entry.metadata() {
        Ok(m) => m,
        Err(_) => return 0,
    };

    let entry_path = entry.path();
    let full_path = entry_path.to_string_lossy().to_string();
    let extension = entry_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(String::from);

    // Times
    let created_str = metadata.created().ok().and_then(system_time_to_string);
    let modified_str = metadata.modified().ok().and_then(system_time_to_string);

    // Attempt to fetch ACL info
    let acls = match get_acl_info(&entry_path, debug_mode, None) {
        Ok(a) => Some(a),
        Err(_) => None,
    };

    // Build a record
    let file_meta = FileMetadata {
        name: file_name,
        full_path: full_path.clone(),
        size: Some(metadata.len()),
        extension,
        created: created_str,
        modified: modified_str,
        acls,
        entry_type: if metadata.is_dir() {
            "directory".to_string()
        } else {
            "file".to_string()
        },
    };

    // Write NDJSON
    if let Err(_e) = write_json_line(&file_meta, writer) {
        // Error handling - could be logged if needed
    }
    
    1 // Return 1 for the entry created
}

//========================================================================
// ACL HANDLING FUNCTIONS
//========================================================================

/// Retrieve a parsed list of ACEs from a file/directory path.
/// Includes bitmask translation to common perms (FullControl, Modify, etc.).
fn get_acl_info(path: &PathBuf, debug_mode: bool, window: Option<&Window>) -> WinResult<Vec<AceInfo>> {
    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, format!("Getting ACL info for: {}", path.display()));
        }
    }
    let wide_path = string_to_wide(&path.to_string_lossy());

    let sec_info: u32 = OWNER_SECURITY_INFORMATION.0
        | GROUP_SECURITY_INFORMATION.0
        | DACL_SECURITY_INFORMATION.0;

    // First call to get required buffer size
    let mut buf_size = 0u32;
    let first_call = unsafe {
        GetFileSecurityW(
            PCWSTR(wide_path.as_ptr()),
            sec_info,
            None,
            0,
            &mut buf_size,
        )
    };

    // We expect this call to fail with ERROR_INSUFFICIENT_BUFFER
    if first_call.as_bool() {
        if debug_mode {
            if let Some(win) = window {
                send_log_message(win, format!("Unexpected success with no buffer for {}", path.display()));
            }
        }
        return Ok(Vec::new());
    }

    // Add some padding to the buffer size to handle dynamic security descriptors
    buf_size += 1024;

    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, format!("Allocating buffer of size {} for security descriptor", buf_size));
        }
    }

    // Allocate buffer and make second call
    let mut sd_buffer = vec![0u8; buf_size as usize];
    let second_call = unsafe {
        GetFileSecurityW(
            PCWSTR(wide_path.as_ptr()),
            sec_info,
            Some(PSECURITY_DESCRIPTOR(sd_buffer.as_mut_ptr() as *mut std::ffi::c_void)),
            buf_size,
            &mut buf_size,
        )
    };

    if !second_call.as_bool() {
        let error = windows::core::Error::from_win32();
        if debug_mode {
            if let Some(win) = window {
                send_log_message(win, format!("Failed to get security descriptor for {}: {:?}", path.display(), error));
            }
        }
        return Ok(Vec::new());
    }

    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, "Successfully got security descriptor".to_string());
        }
    }

    let p_sd = PSECURITY_DESCRIPTOR(sd_buffer.as_ptr() as *mut std::ffi::c_void);

    let mut dacl_present: i32 = 0;
    let mut dacl_defaulted: i32 = 0;
    let mut p_dacl = null_mut();

    let get_dacl_result = unsafe {
        GetSecurityDescriptorDacl(
            p_sd,
            &mut dacl_present as *mut i32 as *mut _,
            &mut p_dacl,
            &mut dacl_defaulted as *mut i32 as *mut _,
        )
    };

    if get_dacl_result.is_err() {
        if debug_mode {
            if let Some(win) = window {
                send_log_message(win, format!("Failed to get DACL for {}: {:?}", path.display(), windows::core::Error::from_win32()));
            }
        }
        return Ok(Vec::new());
    }

    // Check if the DACL is present
    if dacl_present == 0 || p_dacl.is_null() {
        if debug_mode {
            if let Some(win) = window {
                send_log_message(win, format!("No DACL present for {}", path.display()));
            }
        }
        return Ok(Vec::new());
    }

    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, "DACL present, processing ACEs".to_string());
        }
    }

    // p_dacl is an ACL pointer. We'll read its AceCount field:
    let acl_ref = unsafe { &*(p_dacl as *const windows::Win32::Security::ACL) };
    let ace_count = acl_ref.AceCount;
    let mut ace_infos = Vec::with_capacity(ace_count as usize);

    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, format!("Found {} ACEs to process", ace_count));
        }
    }

    for i in 0..ace_count {
        let mut p_ace: *mut std::ffi::c_void = null_mut();
        let get_ace_res = unsafe { GetAce(p_dacl, i as u32, &mut p_ace) };
        if get_ace_res.is_err() || p_ace.is_null() {
            if debug_mode {
                if let Some(win) = window {
                    send_log_message(win, format!("Failed to get ACE {} for {}: {:?}", i, path.display(), windows::core::Error::from_win32()));
                }
            }
            continue;
        }

        let ace_header = unsafe { &*(p_ace as *const ACE_HEADER) };
        if debug_mode {
            if let Some(win) = window {
                send_log_message(win, format!("Processing ACE {} of type {}", i, ace_header.AceType));
            }
        }

        match ace_header.AceType {
            // ACCESS_ALLOWED_ACE_TYPE = 0x00
            0x00 => {
                let allowed_ace = unsafe { &*(p_ace as *const ACCESS_ALLOWED_ACE) };
                let mask = allowed_ace.Mask;
                let sid_ptr = PSID(&allowed_ace.SidStart as *const _ as *mut std::ffi::c_void);
                
                // Just directly convert SID to string with no translation
                let sid_string = sid_to_string_sid(sid_ptr).unwrap_or_else(|| "<INVALID SID>".to_string());
                
                let permissions = parse_access_mask(mask);
                ace_infos.push(AceInfo {
                    identity: sid_string,
                    ace_type: "ALLOWED".to_string(),
                    access_mask: mask,
                    permissions,
                });
            }
            // ACCESS_DENIED_ACE_TYPE = 0x01
            0x01 => {
                let denied_ace = unsafe { &*(p_ace as *const ACCESS_DENIED_ACE) };
                let mask = denied_ace.Mask;
                let sid_ptr = PSID(&denied_ace.SidStart as *const _ as *mut std::ffi::c_void);
                
                // Just directly convert SID to string with no translation
                let sid_string = sid_to_string_sid(sid_ptr).unwrap_or_else(|| "<INVALID SID>".to_string());
                
                let permissions = parse_access_mask(mask);
                ace_infos.push(AceInfo {
                    identity: sid_string,
                    ace_type: "DENIED".to_string(),
                    access_mask: mask,
                    permissions,
                });
            }
            _ => {
                if debug_mode {
                    if let Some(win) = window {
                        send_log_message(win, format!("Skipping ACE type {}", ace_header.AceType));
                    }
                }
            }
        }
    }

    if debug_mode {
        if let Some(win) = window {
            send_log_message(win, format!("Finished processing ACL for {}", path.display()));
        }
    }
    Ok(ace_infos)
}

/// Parse the raw ACE mask bits into friendly strings ("FullControl", "Modify", etc.).
fn parse_access_mask(mask: u32) -> Vec<String> {
    static SIMPLE_PERMISSIONS: &[(u32, &str)] = &[
        (0x1f01ff, "FullControl"),
        (0x0301bf, "Modify"),
        (0x0200a9, "ReadAndExecute"),
        (0x02019f, "ReadAndWrite"),
        (0x020089, "Read"),
        (0x000116, "Write"),
    ];

    // 1) Check exact matches first
    for (bits, name) in SIMPLE_PERMISSIONS {
        if mask == *bits {
            return vec![name.to_string()];
        }
    }

    // 2) Otherwise, check individual bits
    static ACCESS_MASK_BITS: &[(u32, &str)] = &[
        (0x80000000, "GenericRead"),
        (0x40000000, "GenericWrite"),
        (0x20000000, "GenericExecute"),
        (0x10000000, "GenericAll"),
        (0x02000000, "MaximumAllowed"),
        (0x01000000, "AccessSystemSecurity"),
        (0x00100000, "Synchronize"),
        (0x00080000, "WriteOwner"),
        (0x00040000, "WriteDAC"),
        (0x00020000, "ReadControl"),
        (0x00010000, "Delete"),
        (0x00000100, "WriteAttributes"),
        (0x00000080, "ReadAttributes"),
        (0x00000040, "DeleteChild"),
        (0x00000020, "Execute/Traverse"),
        (0x00000010, "WriteExtendedAttributes"),
        (0x00000008, "ReadExtendedAttributes"),
        (0x00000004, "AppendData/AddSubdirectory"),
        (0x00000002, "WriteData/AddFile"),
        (0x00000001, "ReadData/ListDirectory"),
    ];

    let mut perms = Vec::new();
    for (bit, desc) in ACCESS_MASK_BITS {
        if (mask & bit) != 0 {
            perms.push(desc.to_string());
        }
    }
    perms
}

//========================================================================
// UTILITY FUNCTIONS
//========================================================================

/// Write one record as JSON (one line) and flush.
fn write_json_line<T: Serialize>(
    record: &T,
    writer: &mut BufWriter<std::fs::File>,
) -> std::io::Result<()> {
    serde_json::to_writer(&mut *writer, record)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

/// Convert the binary SID to "S-1-5-XX" form using ConvertSidToStringSidW.
fn sid_to_string_sid(sid: PSID) -> Option<String> {
    if sid.0.is_null() {
        return None;
    }

    let mut sid_str_ptr: *mut u16 = std::ptr::null_mut();
    let success = unsafe { ConvertSidToStringSidW(sid, &mut sid_str_ptr) };
    if success == 0 || sid_str_ptr.is_null() {
        return None;
    }

    let s = wide_str_to_string(sid_str_ptr);
    unsafe {
        LocalFree(sid_str_ptr as isize);
    }

    Some(s)
}

/// Convert a SystemTime to a human-readable string (UTC).
fn system_time_to_string(time: SystemTime) -> Option<String> {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    Some(datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

/// Convert `&str` to wide string, null-terminated.
fn string_to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert a wide pointer to a Rust `String`.
fn wide_str_to_string(wide_ptr: *const u16) -> String {
    if wide_ptr.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0;
        while *wide_ptr.add(len) != 0 {
            len += 1;
        }
        let slice = slice::from_raw_parts(wide_ptr, len);
        String::from_utf16_lossy(slice)
    }
}


/// Execute in share enumeration only mode
fn run_share_enum_only_mode(window: &Window, config: &Config, hosts: Vec<String>) -> std::io::Result<()> {
    send_log_message(window, "Mode: Share enumeration only".to_string());
    send_log_message(window, format!("Output file: {}", config.output_path));
    if config.debug_mode {
        send_log_message(window, "Debug mode enabled".to_string());
    }
    
    // Open text file in write mode
    let mut writer = create_output_writer(&config.output_path)?;
    
    // Process hosts for share enumeration only
    for (index, host) in hosts.iter().enumerate() {
        let host = host.trim();
        if host.is_empty() {
            continue;
        }
        
        send_progress_update(
            window,
            format!("Enumerating shares on host: {}", host),
            index,
            Some(hosts.len()),
            "enumerating".to_string(),
        );
        
        send_log_message(window, format!("--- Enumerating shares on host: {} ---", host));
        match enumerate_shares(host) {
            Ok(shares) => {
                for share in shares {
                    // Skip standard admin shares
                    if should_skip_share(&share) {
                        if config.debug_mode {
                            send_log_message(window, format!("Skipping share: {}", share));
                        }
                        continue;
                    }

                    // Write full UNC path to file
                    let unc = format!(r"\\{}\{}", host, share);
                    if let Err(e) = writeln!(writer, "{}", unc) {
                        if config.debug_mode {
                            send_log_message(window, format!("Failed to write share path {}: {:?}", unc, e));
                        }
                    } else {
                        send_log_message(window, format!("[+] Found share: {}", unc));
                    }
                }
            }
            Err(e) => {
                if config.debug_mode {
                    send_log_message(window, format!("Failed to enumerate shares on {}: {:?}", host, e));
                } else {
                    send_log_message(window, format!("Host {} unreachable", host));
                }
            }
        }
    }
    
    // Final flush
    writer.flush()?;
    send_progress_update(
        window,
        "Share enumeration completed".to_string(),
        hosts.len(),
        Some(hosts.len()),
        "complete".to_string(),
    );
    send_log_message(window, format!("Done! Share enumeration written to {}", config.output_path));
    Ok(())
}

/// Execute normal mode with file/directory indexing
fn run_normal_mode(window: &Window, config: &Config, hosts: Vec<String>, shares_to_walk: Vec<String>) -> std::io::Result<usize> {
    send_log_message(window, format!("Max depth: {}", config.max_depth));
    send_log_message(window, format!("Output NDJSON: {}", config.output_path));
    if let Some(limit) = config.max_entries {
        send_log_message(window, format!("Max entries per share: {}", limit));
    }
    if config.debug_mode {
        send_log_message(window, "Debug mode enabled".to_string());
    }
    
    // Open NDJSON file in write mode
    let mut writer = create_output_writer(&config.output_path)?;
    let mut total_entries = 0;

    // Process shares based on mode
    if !shares_to_walk.is_empty() {
        // --shares mode: walk pre-defined shares
        send_log_message(window, "Mode: Walking pre-defined shares".to_string());
        for unc_path in shares_to_walk {
            let unc_path = unc_path.trim();
            if unc_path.is_empty() {
                continue;
            }
            send_log_message(window, format!("[+] Walking share: {}", unc_path));
            let mut share_count = 0;
            let entries = walk_share_unc(window, &unc_path, 0, config.max_depth, config.max_entries, &mut writer, config.debug_mode, &mut share_count);
            total_entries += entries;
            writer.flush()?;
        }
    } else {
        // Normal mode: enumerate shares from hosts
        for host in hosts {
            let host = host.trim();
            if host.is_empty() {
                continue;
            }
            send_log_message(window, format!("--- Enumerating shares on host: {} ---", host));
            
            send_progress_update(
                window,
                format!("Connecting to host: {}", host),
                total_entries,
                None,
                "connecting".to_string(),
            );
            
            match enumerate_shares(host) {
                Ok(shares) => {
                    for share in shares {
                        // Skip standard admin shares
                        if should_skip_share(&share) {
                            if config.debug_mode {
                                send_log_message(window, format!("Skipping share: {}", share));
                            }
                            continue;
                        }

                        // UNC path
                        let unc = format!(r"\\{}\{}", host, share);
                        send_log_message(window, format!("[+] Walking share: {}", unc));
                        let mut share_count = 0;
                        let entries = walk_share_unc(window, &unc, 0, config.max_depth, config.max_entries, &mut writer, config.debug_mode, &mut share_count);
                        total_entries += entries;
                        writer.flush()?;
                    }
                }
                Err(e) => {
                    if config.debug_mode {
                        send_log_message(window, format!("Failed to enumerate shares on {}: {:?}", host, e));
                    } else {
                        send_log_message(window, format!("Host {} unreachable", host));
                    }
                }
            }
        }
    }

    // Final flush
    writer.flush()?;
    send_progress_update(
        window,
        "File enumeration completed".to_string(),
        total_entries,
        None,
        "complete".to_string(),
    );
    send_log_message(window, format!("Done! File enumeration written to {} ({} entries)", config.output_path, total_entries));
    Ok(total_entries)
}

//========================================================================
// THREADED WALKER FUNCTIONS
//========================================================================

/// Threaded version of process_share_root — sends JSON via channel
fn process_share_root_threaded(unc_path: &str, sender: &Sender<WorkerOutput>, debug_mode: bool) -> usize {
    let path = PathBuf::from(unc_path);
    match fs::metadata(&path) {
        Ok(metadata) => {
            let created_str = metadata.created().ok().and_then(system_time_to_string);
            let modified_str = metadata.modified().ok().and_then(system_time_to_string);

            let acls = match get_acl_info(&path, debug_mode, None) {
                Ok(a) => Some(a),
                Err(_) => None,
            };

            let share_meta = FileMetadata {
                name: unc_path.to_string(),
                full_path: unc_path.to_string(),
                size: None,
                extension: None,
                created: created_str,
                modified: modified_str,
                acls,
                entry_type: "share".to_string(),
            };

            if let Ok(json) = serde_json::to_string(&share_meta) {
                let _ = sender.send(WorkerOutput::JsonlRecord(json));
            }
            1
        }
        Err(_) => 0,
    }
}

/// Threaded version of process_filesystem_entry — sends JSON via channel
fn process_filesystem_entry_threaded(
    entry: &fs::DirEntry,
    sender: &Sender<WorkerOutput>,
    debug_mode: bool,
) -> usize {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let metadata = match entry.metadata() {
        Ok(m) => m,
        Err(_) => return 0,
    };

    let entry_path = entry.path();
    let full_path = entry_path.to_string_lossy().to_string();
    let extension = entry_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(String::from);

    let created_str = metadata.created().ok().and_then(system_time_to_string);
    let modified_str = metadata.modified().ok().and_then(system_time_to_string);

    let acls = match get_acl_info(&entry_path, debug_mode, None) {
        Ok(a) => Some(a),
        Err(_) => None,
    };

    let file_meta = FileMetadata {
        name: file_name,
        full_path: full_path.clone(),
        size: Some(metadata.len()),
        extension,
        created: created_str,
        modified: modified_str,
        acls,
        entry_type: if metadata.is_dir() {
            "directory".to_string()
        } else {
            "file".to_string()
        },
    };

    if let Ok(json) = serde_json::to_string(&file_meta) {
        let _ = sender.send(WorkerOutput::JsonlRecord(json));
    }
    1
}

/// Threaded version of walk_share_unc — sends results via channel, checks abort flag
/// Uses a local per-share counter for max_entries so the limit applies per-share, not globally.
fn walk_share_unc_threaded(
    unc_path: &str,
    current_depth: usize,
    max_depth: usize,
    max_entries: Option<usize>,
    sender: &Sender<WorkerOutput>,
    debug_mode: bool,
    global_count: &AtomicUsize,
) -> usize {
    let mut entries_count = 0;

    if current_depth == 0 {
        let added = process_share_root_threaded(unc_path, sender, debug_mode);
        entries_count += added;
        global_count.fetch_add(added, Ordering::Relaxed);
    }

    if current_depth > max_depth {
        return entries_count;
    }

    if INDEX_ABORT_REQUESTED.load(Ordering::Relaxed) {
        return entries_count;
    }

    if let Some(limit) = max_entries {
        if entries_count >= limit {
            return entries_count;
        }
    }

    let path = PathBuf::from(unc_path);
    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(_) => return entries_count,
    };

    for entry_result in entries {
        if INDEX_ABORT_REQUESTED.load(Ordering::Relaxed) {
            return entries_count;
        }

        if let Some(limit) = max_entries {
            if entries_count >= limit {
                if current_depth == 0 {
                    let _ = sender.send(WorkerOutput::Log(format!(
                        "Reached max entries limit ({}) for share: {}", limit, unc_path
                    )));
                }
                return entries_count;
            }
        }

        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let added = process_filesystem_entry_threaded(&entry, sender, debug_mode);
        entries_count += added;
        global_count.fetch_add(added, Ordering::Relaxed);

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            let full_path = entry.path().to_string_lossy().to_string();
            let sub_entries = walk_share_unc_threaded(
                &full_path,
                current_depth + 1,
                max_depth,
                max_entries,
                sender,
                debug_mode,
                global_count,
            );
            entries_count += sub_entries;

            // Check again after recursion
            if let Some(limit) = max_entries {
                if entries_count >= limit {
                    if current_depth == 0 {
                        let _ = sender.send(WorkerOutput::Log(format!(
                            "Reached max entries limit ({}) after recursion for share: {}", limit, unc_path
                        )));
                    }
                    return entries_count;
                }
            }
        }
    }

    if current_depth == 0 {
        let _ = sender.send(WorkerOutput::Log(format!(
            "Completed share: {} ({} entries)", unc_path, entries_count
        )));
    }

    entries_count
}

//========================================================================
// WORKER AND WRITER THREAD FUNCTIONS
//========================================================================

/// Worker thread: pulls work items from queue, processes them, sends results
fn worker_thread(
    worker_id: usize,
    queue: &WorkQueue,
    sender: Sender<WorkerOutput>,
    config: &WorkerConfig,
    global_count: Arc<AtomicUsize>,
    share_enum_only: bool,
) {
    // Set up per-thread impersonation
    let _impersonation_guard = if !config.smb_username.is_empty() && !config.smb_password.is_empty() {
        match crate::smb_auth::start_impersonation(&config.smb_username, &config.smb_password, &config.smb_domain) {
            Ok(guard) => {
                let _ = sender.send(WorkerOutput::Log(format!(
                    "Worker {}: SMB impersonation established", worker_id
                )));
                Some(guard)
            }
            Err(e) => {
                let _ = sender.send(WorkerOutput::Error(format!(
                    "Worker {}: Failed to establish SMB impersonation: {}", worker_id, e
                )));
                let _ = sender.send(WorkerOutput::Done);
                return;
            }
        }
    } else {
        None
    };

    while !INDEX_ABORT_REQUESTED.load(Ordering::Relaxed) {
        let item = match queue.next() {
            Some(i) => i,
            None => break,
        };
        let item = item.trim().to_string();
        if item.is_empty() {
            continue;
        }

        if share_enum_only {
            // Mode 0: enumerate shares on host, send TextLine per share
            let _ = sender.send(WorkerOutput::Log(format!(
                "Worker {}: Enumerating shares on host: {}", worker_id, item
            )));
            match enumerate_shares(&item) {
                Ok(shares) => {
                    for share in shares {
                        if should_skip_share(&share) {
                            if config.debug_mode {
                                let _ = sender.send(WorkerOutput::Log(format!(
                                    "Worker {}: Skipping share: {}", worker_id, share
                                )));
                            }
                            continue;
                        }
                        let unc = format!(r"\\{}\{}", item, share);
                        let _ = sender.send(WorkerOutput::TextLine(unc.clone()));
                        let _ = sender.send(WorkerOutput::Log(format!(
                            "[+] Worker {}: Found share: {}", worker_id, unc
                        )));
                    }
                }
                Err(e) => {
                    let msg = if config.debug_mode {
                        format!("Worker {}: Failed to enumerate shares on {}: {:?}", worker_id, item, e)
                    } else {
                        format!("Worker {}: Host {} unreachable", worker_id, item)
                    };
                    let _ = sender.send(WorkerOutput::Error(msg));
                }
            }
        } else {
            // Mode 1/2: walk a UNC share path
            let _ = sender.send(WorkerOutput::Log(format!(
                "Worker {}: Walking share: {}", worker_id, item
            )));
            walk_share_unc_threaded(
                &item,
                0,
                config.max_depth,
                config.max_entries,
                &sender,
                config.debug_mode,
                &global_count,
            );
        }
    }

    let _ = sender.send(WorkerOutput::Done);
}

/// Writer thread: receives WorkerOutput, writes to file, emits progress to window
fn writer_thread(
    receiver: Receiver<WorkerOutput>,
    window: Window,
    output_path: String,
    total_workers: usize,
    _share_enum_only: bool,
) -> (usize, Vec<String>) {
    let file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&output_path)
    {
        Ok(f) => f,
        Err(e) => {
            let _ = window.emit("indexing-log", format!("Writer: Failed to open output file: {}", e));
            return (0, vec![e.to_string()]);
        }
    };

    let mut writer = BufWriter::new(file);
    let mut total_entries: usize = 0;
    let mut errors = Vec::new();
    let mut done_count = 0;
    let mut records_since_flush: usize = 0;

    while done_count < total_workers {
        if INDEX_ABORT_REQUESTED.load(Ordering::Relaxed) {
            break;
        }

        match receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(msg) => match msg {
                WorkerOutput::JsonlRecord(json) => {
                    let _ = writer.write_all(json.as_bytes());
                    let _ = writer.write_all(b"\n");
                    total_entries += 1;
                    records_since_flush += 1;

                    if records_since_flush >= 50 {
                        let _ = writer.flush();
                        records_since_flush = 0;
                    }

                    let _ = window.emit("indexing-progress", ProgressUpdate {
                        message: format!("Processed {} entries", total_entries),
                        current: total_entries,
                        total: None,
                        stage: "walking".to_string(),
                    });
                }
                WorkerOutput::TextLine(line) => {
                    let _ = writeln!(writer, "{}", line);
                    total_entries += 1;
                    records_since_flush += 1;

                    if records_since_flush >= 50 {
                        let _ = writer.flush();
                        records_since_flush = 0;
                    }

                    let _ = window.emit("indexing-progress", ProgressUpdate {
                        message: format!("Found {} shares", total_entries),
                        current: total_entries,
                        total: None,
                        stage: "enumerating".to_string(),
                    });
                }
                WorkerOutput::Log(msg) => {
                    let _ = window.emit("indexing-log", msg);
                }
                WorkerOutput::Error(msg) => {
                    let _ = window.emit("indexing-log", msg.clone());
                    errors.push(msg);
                }
                WorkerOutput::Done => {
                    done_count += 1;
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Final flush
    let _ = writer.flush();

    (total_entries, errors)
}

//========================================================================
// TAURI COMMANDS
//========================================================================

/// Start the active indexing process
#[tauri::command]
pub async fn start_active_indexing(
    window: Window,
    config: IndexConfig,
) -> Result<IndexResult, String> {
    // Reset abort flag
    INDEX_ABORT_REQUESTED.store(false, Ordering::Relaxed);

    // Send initial progress
    send_progress_update(
        &window,
        "Starting active indexing...".to_string(),
        0,
        None,
        "connecting".to_string(),
    );

    // Create output file in current directory
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let output_filename = if config.share_enum_only {
        format!("enumerated_shares_{}.txt", timestamp)
    } else {
        format!("indexed_shares_{}.jsonl", timestamp)
    };

    // Extract SMB credentials before moving config
    let smb_username = config.smb_username.clone().unwrap_or_default();
    let smb_password = config.smb_password.clone().unwrap_or_default();
    let smb_domain = config.smb_domain.clone().unwrap_or_default();
    let thread_count = config.thread_count.unwrap_or(4).max(1).min(32);
    let internal_config = Config::from_index_config(config, output_filename.clone());

    // Send log message about configuration
    send_log_message(&window, "Active indexing started".to_string());
    send_log_message(&window, format!("Max depth: {}", internal_config.max_depth));
    send_log_message(&window, format!("Output file: {}", output_filename));
    send_log_message(&window, format!("Thread count: {}", thread_count));
    if let Some(limit) = internal_config.max_entries {
        send_log_message(&window, format!("Max entries per share: {}", limit));
    }
    if internal_config.debug_mode {
        send_log_message(&window, "Debug mode enabled".to_string());
    }

    // Handle different input modes
    let (hosts, shares_to_walk) = if let Some(shares_file) = &internal_config.shares_file {
        let shares = load_shares_from_file(shares_file);
        if internal_config.debug_mode {
            send_log_message(&window, format!("Loaded {} shares from file: {}", shares.len(), shares_file));
        }
        (Vec::new(), shares)
    } else {
        let hosts = if internal_config.target_or_file.contains(',') {
            internal_config.target_or_file
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            load_hosts(&internal_config.target_or_file)
        };
        (hosts, Vec::new())
    };

    if internal_config.shares_file.is_some() {
        send_log_message(&window, format!("Shares to walk: {}", shares_to_walk.len()));
    } else {
        send_log_message(&window, format!("Hosts to enumerate: {}", hosts.len()));
    }

    let _aborted = INDEX_ABORT_REQUESTED.load(Ordering::Relaxed);
    let total_entries;
    let errors;

    if thread_count == 1 {
        // Sequential fallback (thread_count == 1)
        let _impersonation_guard = if !smb_username.is_empty() && !smb_password.is_empty() {
            send_log_message(&window, format!(
                "Using explicit SMB credentials: {}{}{}",
                if smb_domain.is_empty() { "" } else { &smb_domain },
                if smb_domain.is_empty() { "" } else { "\\" },
                &smb_username
            ));
            match crate::smb_auth::start_impersonation(&smb_username, &smb_password, &smb_domain) {
                Ok(guard) => {
                    send_log_message(&window, "SMB impersonation established successfully".to_string());
                    Some(guard)
                }
                Err(e) => {
                    send_log_message(&window, format!("Failed to establish SMB impersonation: {}", e));
                    return Err(format!("SMB authentication failed: {}", e));
                }
            }
        } else {
            send_log_message(&window, "Using current session credentials".to_string());
            None
        };

        let mut errs = Vec::new();
        let entries = if internal_config.share_enum_only {
            match run_share_enum_only_mode(&window, &internal_config, hosts) {
                Ok(_) => 0,
                Err(e) => {
                    errs.push(e.to_string());
                    0
                }
            }
        } else {
            match run_normal_mode(&window, &internal_config, hosts, shares_to_walk) {
                Ok(e) => e,
                Err(e) => {
                    errs.push(e.to_string());
                    0
                }
            }
        };
        total_entries = entries;
        errors = errs;
    } else {
        // Threaded path
        // Phase 1: enumerate all shares sequentially (for Mode 0/1)
        let work_items = if internal_config.share_enum_only {
            // Mode 0: work items are hosts
            hosts
        } else if !shares_to_walk.is_empty() {
            // Mode 2: work items are UNC paths from file
            shares_to_walk
        } else {
            // Mode 1: enumerate shares from hosts, collect UNC paths
            let _impersonation_guard = if !smb_username.is_empty() && !smb_password.is_empty() {
                send_log_message(&window, format!(
                    "Using explicit SMB credentials: {}{}{}",
                    if smb_domain.is_empty() { "" } else { &smb_domain },
                    if smb_domain.is_empty() { "" } else { "\\" },
                    &smb_username
                ));
                match crate::smb_auth::start_impersonation(&smb_username, &smb_password, &smb_domain) {
                    Ok(guard) => {
                        send_log_message(&window, "SMB impersonation established for Phase 1".to_string());
                        Some(guard)
                    }
                    Err(e) => {
                        send_log_message(&window, format!("Failed to establish SMB impersonation: {}", e));
                        return Err(format!("SMB authentication failed: {}", e));
                    }
                }
            } else {
                send_log_message(&window, "Using current session credentials".to_string());
                None
            };

            let mut all_uncs = Vec::new();
            for host in &hosts {
                if INDEX_ABORT_REQUESTED.load(Ordering::Relaxed) {
                    break;
                }
                let host = host.trim();
                if host.is_empty() {
                    continue;
                }
                send_log_message(&window, format!("Enumerating shares on host: {}", host));
                match enumerate_shares(host) {
                    Ok(shares) => {
                        for share in shares {
                            if should_skip_share(&share) {
                                continue;
                            }
                            all_uncs.push(format!(r"\\{}\{}", host, share));
                        }
                    }
                    Err(e) => {
                        let msg = if internal_config.debug_mode {
                            format!("Failed to enumerate shares on {}: {:?}", host, e)
                        } else {
                            format!("Host {} unreachable", host)
                        };
                        send_log_message(&window, msg);
                    }
                }
            }
            send_log_message(&window, format!("Phase 1 complete: {} shares to walk", all_uncs.len()));
            all_uncs
        };

        if work_items.is_empty() {
            send_progress_update(&window, "No work items found".to_string(), 0, None, "complete".to_string());
            return Ok(IndexResult {
                success: true,
                message: "No shares or hosts to process".to_string(),
                output_file: output_filename,
                total_entries: 0,
                errors: vec![],
            });
        }

        // Phase 2: spawn workers + writer
        let queue = Arc::new(WorkQueue::new(work_items));
        let global_count = Arc::new(AtomicUsize::new(0));
        let (sender, receiver) = mpsc::channel();

        let worker_config = WorkerConfig {
            max_depth: internal_config.max_depth,
            max_entries: internal_config.max_entries,
            debug_mode: internal_config.debug_mode,
            smb_username,
            smb_password,
            smb_domain,
        };

        // Spawn workers
        let num_workers = thread_count.min(queue.len());
        let mut worker_handles = Vec::with_capacity(num_workers);

        send_log_message(&window, format!("Spawning {} worker threads", num_workers));

        for worker_id in 0..num_workers {
            let sender = sender.clone();
            let queue = Arc::clone(&queue);
            let global_count = Arc::clone(&global_count);
            let config = WorkerConfig {
                max_depth: worker_config.max_depth,
                max_entries: worker_config.max_entries,
                debug_mode: worker_config.debug_mode,
                smb_username: worker_config.smb_username.clone(),
                smb_password: worker_config.smb_password.clone(),
                smb_domain: worker_config.smb_domain.clone(),
            };
            let share_enum_only = internal_config.share_enum_only;

            let handle = thread::spawn(move || {
                worker_thread(worker_id, &queue, sender, &config, global_count, share_enum_only);
            });
            worker_handles.push(handle);
        }

        // Drop the main thread's sender so the writer detects completion
        drop(sender);

        // Spawn writer thread
        let writer_window = window.clone();
        let writer_output = output_filename.clone();
        let writer_share_enum_only = internal_config.share_enum_only;

        let writer_handle = thread::spawn(move || {
            writer_thread(receiver, writer_window, writer_output, num_workers, writer_share_enum_only)
        });

        // Wait for all workers to finish
        for handle in worker_handles {
            let _ = handle.join();
        }

        // Wait for writer to finish
        let (entries, errs) = writer_handle.join().unwrap_or((0, vec!["Writer thread panicked".to_string()]));
        total_entries = entries;
        errors = errs;
    }

    let was_aborted = INDEX_ABORT_REQUESTED.load(Ordering::Relaxed);
    if was_aborted {
        send_log_message(&window, "Indexing aborted by user".to_string());
    }

    send_progress_update(
        &window,
        if was_aborted { "Indexing aborted".to_string() } else { "Indexing completed successfully".to_string() },
        total_entries,
        None,
        "complete".to_string(),
    );

    Ok(IndexResult {
        success: !was_aborted,
        message: if was_aborted {
            "Indexing aborted by user".to_string()
        } else {
            "Active indexing completed successfully".to_string()
        },
        output_file: output_filename,
        total_entries,
        errors,
    })
}

/// Abort the active indexing process
#[tauri::command]
pub async fn abort_active_indexing() -> Result<(), String> {
    INDEX_ABORT_REQUESTED.store(true, Ordering::Relaxed);
    Ok(())
}
