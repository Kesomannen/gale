const COMMANDS: &[&str] = &["query_packages"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
