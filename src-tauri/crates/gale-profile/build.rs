const COMMANDS: &[&str] = &[
    "create",
    "delete",
    "query",
    "rename",
    "force_uninstall_mod",
    "force_toggle_mod",
    "queue_thunderstore_install",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
