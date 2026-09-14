use std::process::Command;

pub fn is_flatpak() -> bool {
    cfg!(target_os = "linux") && std::env::var("container").is_ok()
}

pub fn wrap_command(command: &Command) -> Command {
    let mut wrapper = Command::new("flatpak-spawn");
    wrapper
        .arg("--host")
        .arg(command.get_program())
        .args(command.get_args());

    wrapper
}

/// If running inside flatpak, wraps the command in a
/// `flatpak-spawn --host` command to run it outside the sandbox.
pub fn wrap_command_if_needed(command: &mut Command) {
    if is_flatpak() {
        *command = wrap_command(command);
    }
}
