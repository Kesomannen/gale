use crate::{
    manager::ModManager,
    prefs::Prefs,
    util::cmd::{Result, StateMutex},
};

#[tauri::command]
pub fn launch_game(
    vanilla: bool,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game().launch(vanilla, &prefs)?;
    Ok(())
}
