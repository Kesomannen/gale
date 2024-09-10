const COMMANDS: &[&str] = &["create", "delete", "query", "rename"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
