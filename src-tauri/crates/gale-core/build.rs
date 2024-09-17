const COMMANDS: &[&str] = &["get_games"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
