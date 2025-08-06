// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod json_importer;
mod saved_queries;
mod active_index_shares;
mod active_index_ldap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            json_importer::import_json_to_neo4j,
            json_importer::get_file_info,
            saved_queries::get_saved_queries,
            saved_queries::add_user_query,
            saved_queries::delete_user_query,
            active_index_shares::start_active_indexing,
            active_index_ldap::start_ldap_enumeration
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
