use std::fs::{self, File};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Window};

// ============================
// Config and Result Structures
// ============================

#[derive(Debug, Deserialize)]
pub struct CredGatherConfig {
    pub file_list: String,
    pub string_list: String,
    pub debug_mode: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProgressUpdate {
    pub message: String,
    pub current: usize,
    pub total: Option<usize>,
    pub stage: String, // "starting", "scanning", "complete", "error"
}

#[derive(Debug, Serialize)]
pub struct GatherResult {
    pub success: bool,
    pub message: String,
    pub output_file: String,
    pub total_entries: usize,
    pub errors: Vec<String>,
}

// ============================
// Event helpers
// ============================

fn send_progress_update(window: &Window, message: String, current: usize, total: Option<usize>, stage: &str) {
    let update = ProgressUpdate {
        message,
        current,
        total,
        stage: stage.to_string(),
    };
    let _ = window.emit("credential-gathering-progress", update);
}

fn send_log_message(window: &Window, message: String) {
    let _ = window.emit("credential-gathering-log", message);
}

// ============================
// Abort coordination
// ============================

static ABORT_REQUESTED: AtomicBool = AtomicBool::new(false);

// ============================
// Core helpers
// ============================

fn read_lines(filename: &str) -> Result<Vec<String>, String> {
    let file = File::open(filename).map_err(|e| format!("Failed to open file {}: {}", filename, e))?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(content) => {
                let trimmed = content.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
        }
    }
            Err(e) => return Err(format!("Failed to read line in {}: {}", filename, e)),
        }
    }
    Ok(lines)
}

fn search_file(file_path: &str, search_strings: &[String]) -> Result<Vec<String>, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
    let reader = BufReader::new(file);

    // Read all lines to include context lines
    let lines: Vec<String> = reader
        .lines()
        .map(|l| l.unwrap_or_default())
        .collect();

    let mut hits = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        // Look for any search string in this line
        if search_strings.iter().any(|s| line.contains(s)) {
                let mut context = String::new();
                
            // Line before
                if line_num > 0 {
                    let before_line = highlight_search_strings(&lines[line_num - 1], search_strings);
                    context.push_str(&format!(
                        "<div class=\"line line-before\"><span class=\"line-number\">{}</span>{}</div>",
                        line_num,
                        before_line
                    ));
                }
                
            // Hit line
                let highlighted_line = highlight_search_strings(line, search_strings);
                context.push_str(&format!(
                "<div class=\"line line-hit\"><span class=\"line-number\">{}</span>{} <span class=\"hit-marker\"></span></div>",
                    line_num + 1,
                    highlighted_line
                ));
                
            // Line after
            if line_num + 1 < lines.len() {
                    let after_line = highlight_search_strings(&lines[line_num + 1], search_strings);
                    context.push_str(&format!(
                        "<div class=\"line line-after\"><span class=\"line-number\">{}</span>{}</div>",
                        line_num + 2,
                        after_line
                    ));
                }
                
                hits.push(context);
        }
    }

    Ok(hits)
}

fn generate_html_report(
    file_results: &[(String, Vec<String>)],
    search_strings: &[String],
    total_files_processed: usize,
    total_files_with_hits: usize,
) -> Result<String, String> {
    let template = r#"<!DOCTYPE html>
<html lang='en'>
<head>
    <meta charset='UTF-8'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0'>
    <title>Pillage Suite - Credential Gathering Results</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
            margin-bottom: 30px;
        }
        .summary {
            background-color: #e8f4fd;
            padding: 15px;
            border-radius: 5px;
            margin-bottom: 20px;
        }
        .file-entry {
            margin-bottom: 30px;
            border: 1px solid #ddd;
            border-radius: 5px;
            overflow: hidden;
        }
        .file-header {
            background-color: #f8f9fa;
            padding: 10px 15px;
            border-bottom: 1px solid #ddd;
            font-weight: bold;
            color: #495057;
            cursor: pointer;
        }
        .file-content {
            padding: 15px;
            background-color: #fafafa;
        }
        .line {
            font-family: 'Courier New', monospace;
            margin: 2px 0;
            padding: 2px 5px;
            border-radius: 3px;
        }
        .line-hit {
            background-color: #fff3cd;
            border-left: 4px solid #ffc107;
        }
        .line-before, .line-after {
            background-color: #f8f9fa;
            color: #6c757d;
        }
        .highlight {
            background-color: #ffeb3b;
            padding: 1px 2px;
            border-radius: 2px;
            font-weight: bold;
        }
        .line-number {
            color: #6c757d;
            font-weight: normal;
            margin-right: 10px;
        }
        .hit-marker {
            color: #dc3545;
            font-weight: bold;
        }
        .app-logo {
            width: 48px;
            height: 48px;
            margin-right: 15px;
            color: #1e293b;
        }
        .title-container {
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 30px;
        }
        .footer {
            margin-top: 40px;
            padding: 20px;
            background-color: #f8f9fa;
            border-top: 1px solid #dee2e6;
            text-align: center;
            color: #6c757d;
            font-size: 14px;
        }
        .footer a {
            color: #495057;
            text-decoration: none;
        }
        .footer a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class='container'>
        <div class='title-container'>
            <svg class='app-logo' viewBox='0 0 24 24' fill='none' xmlns='http://www.w3.org/2000/svg'>
                <path d='M22 5h-9l-2-2H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h19c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2z' transform='scale(0.92) translate(1, 1)' fill='currentColor'/>
                <g transform='translate(6.5, 7) scale(0.025)' fill='#e8f4fd' fill-opacity='0.8'>
                    <path d='M339.588,314.529c-14.215,0-27.456,4.133-38.621,11.239l-112.682-78.67c1.809-6.315,2.798-12.976,2.798-19.871    c0-6.896-0.989-13.557-2.798-19.871l109.64-76.547c11.764,8.356,26.133,13.286,41.662,13.286c39.79,0,72.047-32.257,72.047-72.047    C411.634,32.258,379.378,0,339.588,0c-39.79,0-72.047,32.257-72.047,72.047c0,5.255,0.578,10.373,1.646,15.308l-112.424,78.491    c-10.974-6.759-23.892-10.666-37.727-10.666c-39.79,0-72.047,32.257-72.047,72.047s32.256,72.047,72.047,72.047    c13.834,0,26.753-3.907,37.727-10.666l113.292,79.097c-1.629,6.017-2.514,12.34-2.514,18.872c0,39.79,32.257,72.047,72.047,72.047    c39.79,0,72.047-32.257,72.047-72.047C411.635,346.787,379.378,314.529,339.588,314.529z'/>
                </g>
            </svg>
            <h1>Pillage Suite - Credential Gathering Results</h1>
        </div>
        <div class='summary'>
            <h3>Summary</h3>
            <p><strong>Total files processed:</strong> {{TOTAL_FILES_PROCESSED}}</p>
            <p><strong>Files with hits:</strong> {{FILES_WITH_HITS}}</p>
            <p><strong>Search strings:</strong> {{SEARCH_STRINGS}}</p>
            <p><strong>Generated:</strong> {{GENERATED_TIMESTAMP}}</p>
        </div>
        {{FILE_ENTRIES}}
    </div>
    <div class='footer'>This report was created with <strong>Pillage Suite</strong> - <a href='https://github.com/m0xr4/PillageSuite' target='_blank'>https://github.com/m0xr4/PillageSuite</a></div>
    </body>
    </html>
"#;

    let mut file_entries_html = String::new();
    for (file_path, hits) in file_results {
        file_entries_html.push_str(&format!(
            r#"<div class="file-entry">
    <details open>
      <summary class="file-header">{}</summary>
                <div class="file-content">"#,
            html_escape(file_path)
        ));

        for hit in hits {
            file_entries_html.push_str(hit);
        }
        file_entries_html.push_str("</div></details></div>");
    }

    let html_content = template
        .replace("{{TOTAL_FILES_PROCESSED}}", &total_files_processed.to_string())
        .replace("{{FILES_WITH_HITS}}", &total_files_with_hits.to_string())
        .replace("{{SEARCH_STRINGS}}", &search_strings.join(", "))
        .replace(
            "{{GENERATED_TIMESTAMP}}",
            &Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        )
        .replace("{{FILE_ENTRIES}}", &file_entries_html);

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let html_filename = format!("search_results_{}.html", timestamp);
    fs::write(&html_filename, html_content)
        .map_err(|e| format!("Failed to write HTML file {}: {}", html_filename, e))?;

    Ok(html_filename)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn highlight_search_strings(line: &str, search_strings: &[String]) -> String {
    let mut highlighted_line = html_escape(line);
    for s in search_strings {
        let escaped = html_escape(s);
        if highlighted_line.contains(&escaped) {
            let replacement = format!("<span class=\"highlight\">{}</span>", escaped);
            highlighted_line = highlighted_line.replace(&escaped, &replacement);
        }
    }
    highlighted_line
}

// ============================
// Tauri command
// ============================

#[tauri::command]
pub async fn start_credential_gathering(window: Window, config: CredGatherConfig) -> Result<GatherResult, String> {
    // reset any previous abort request at the start
    ABORT_REQUESTED.store(false, Ordering::Relaxed);
    send_progress_update(&window, "Starting credential gathering...".to_string(), 0, None, "starting");
    send_log_message(&window, "Credential gathering started".to_string());

    // Validate input files exist
    if !Path::new(&config.file_list).exists() {
        let msg = format!("Target file list not found: {}", config.file_list);
        send_log_message(&window, msg.clone());
        send_progress_update(&window, msg.clone(), 0, None, "error");
        return Ok(GatherResult { success: false, message: msg, output_file: String::new(), total_entries: 0, errors: vec!["file_list not found".to_string()] });
    }
    if !Path::new(&config.string_list).exists() {
        let msg = format!("String list not found: {}", config.string_list);
        send_log_message(&window, msg.clone());
        send_progress_update(&window, msg.clone(), 0, None, "error");
        return Ok(GatherResult { success: false, message: msg, output_file: String::new(), total_entries: 0, errors: vec!["string_list not found".to_string()] });
    }

    // Load inputs
    let mut search_strings = match read_lines(&config.string_list) {
        Ok(v) => v,
        Err(e) => {
            send_progress_update(&window, e.clone(), 0, None, "error");
            return Ok(GatherResult { success: false, message: e.clone(), output_file: String::new(), total_entries: 0, errors: vec![e] });
        }
    };
    // Deduplicate search strings while preserving order
    let mut seen = HashSet::new();
    search_strings.retain(|s| seen.insert(s.clone()));
    send_log_message(&window, format!("Loaded {} unique search strings", search_strings.len()));

    let file_paths = match read_lines(&config.file_list) {
        Ok(v) => v,
        Err(e) => {
            send_progress_update(&window, e.clone(), 0, None, "error");
            return Ok(GatherResult { success: false, message: e.clone(), output_file: String::new(), total_entries: 0, errors: vec![e] });
        }
    };
    send_log_message(&window, format!("Loaded {} file paths to search", file_paths.len()));

    let total = file_paths.len();
    send_progress_update(&window, "Scanning files...".to_string(), 0, Some(total), "scanning");

    let mut file_results: Vec<(String, Vec<String>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut current: usize = 0;
    let mut aborted: bool = false;

    for file_path in file_paths {
        if config.debug_mode {
            send_log_message(&window, format!("Processing file: {}", file_path));
        }
        // Check for abort before processing each file
        if ABORT_REQUESTED.load(Ordering::Relaxed) {
            aborted = true;
            send_log_message(&window, "Abort requested. Finalizing partial results...".to_string());
            break;
        }

        match search_file(&file_path, &search_strings) {
            Ok(hits) => {
                if !hits.is_empty() {
                    send_log_message(&window, format!("[+] Hits in: {} ({} matches)", file_path, hits.len()));
                    file_results.push((file_path.clone(), hits));
                }
            }
            Err(e) => {
                let msg = format!("Error processing file {}: {}", file_path, e);
                send_log_message(&window, msg.clone());
                errors.push(msg);
            }
        }

        // Increment after finishing processing the file so the count reflects completed work
        current += 1;
        // Emit progress on every file to ensure the UI updates incrementally
        send_progress_update(&window, format!("Processed {} files", current), current, Some(total), "scanning");
    }

    let total_files_with_hits = file_results.len();
    let total_hit_entries: usize = file_results.iter().map(|(_, hits)| hits.len()).sum();

    // Generate report
    let (success, message, output_file) = if aborted {
        // On abort, always write a report to reflect partial progress
        match generate_html_report(&file_results, &search_strings, total, total_files_with_hits) {
            Ok(path) => {
                send_log_message(&window, format!("HTML report written to: {}", &path));
                (true, "Aborted by user. Partial report generated".to_string(), path)
            }
            Err(e) => {
                let msg = format!("Aborted by user. Failed to write HTML report: {}", e);
                send_log_message(&window, msg.clone());
                (false, msg, String::new())
            }
        }
    } else if !file_results.is_empty() {
        match generate_html_report(&file_results, &search_strings, total, total_files_with_hits) {
            Ok(path) => {
                send_log_message(&window, format!("HTML report written to: {}", &path));
                (true, "Credential gathering completed".to_string(), path)
            }
            Err(e) => {
                let msg = format!("Failed to write HTML report: {}", e);
                send_log_message(&window, msg.clone());
                (false, msg, String::new())
            }
        }
    } else {
        (true, "Completed. No hits found".to_string(), String::new())
    };

    let stage = if aborted { "aborted" } else if success { "complete" } else { "error" };
    let final_current = if aborted { current } else { total };
    send_progress_update(&window, message.clone(), final_current, Some(total), stage);

    Ok(GatherResult {
        success,
        message,
        output_file,
        total_entries: total_hit_entries,
        errors,
    })
}

#[tauri::command]
pub async fn abort_credential_gathering(window: Window) -> Result<(), String> {
    ABORT_REQUESTED.store(true, Ordering::Relaxed);
    send_log_message(&window, "Abort requested by user".to_string());
    // Emit a progress update to reflect the abort intent immediately; total/current unknown here
    send_progress_update(&window, "Abort requested by user".to_string(), 0, None, "aborting");
    Ok(())
}
