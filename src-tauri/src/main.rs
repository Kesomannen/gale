fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(gale_core::init())
        .plugin(gale_thunderstore::init())
        .plugin(gale_profile::init())
        .plugin(gale_io::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
