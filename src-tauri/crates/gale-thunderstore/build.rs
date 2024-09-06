const COMMANDS: &[&str] = &["query_packages", "query_package"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
