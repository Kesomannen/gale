const COMMANDS: &[&str] = &["get_communities"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
