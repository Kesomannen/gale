const COMMANDS: &[&str] = &[
    "read_code",
    "read_file",
    "import",
    "export_file",
    "export_code",
    "export_modpack",
    "publish_modpack",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
