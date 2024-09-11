const COMMANDS: &[&str] = &[
    "create",
    "delete",
    "query",
    "rename",
    "force_uninstall",
    "force_toggle",
    "install_from_thunderstore",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
