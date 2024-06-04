use crate::{
    util::cmd::{Result, StateMutex},
    manager::ModManager,
    prefs::Prefs,
};

#[tauri::command]
pub fn launch_game(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.launch_game(&prefs)?;
    Ok(())
}
