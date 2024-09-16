const COMMANDS: &[&str] = &[
    "read_code",
    "read_file",
    "import",
    "export_file",
    "export_code",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
