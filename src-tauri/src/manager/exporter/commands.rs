use super::ModpackArgs;
use crate::{
    util::cmd::{Result, StateMutex},
    manager::ModManager,
    prefs::Prefs,
    thunderstore::Thunderstore,
    NetworkClient,
};
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn export_code(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<Uuid> {
    let key = super::export_code(&client.0, manager, thunderstore, prefs).await?;

    Ok(key)
}

#[tauri::command]
pub fn export_file(
    mut dir: PathBuf,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    super::export_file(manager.active_profile(), &mut dir, &thunderstore)?;
    let _ = open::that(dir.parent().unwrap());

    Ok(())
}

#[tauri::command]
pub fn export_pack(
    path: PathBuf,
    args: ModpackArgs,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let zip_path = path.join(&args.name).with_extension("zip");
    super::export_pack(manager.active_profile(), &zip_path, args, &thunderstore)?;

    let _ = open::that(&zip_path);
    Ok(())
}

#[tauri::command]
pub fn export_dep_string(
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    manager
        .active_profile()
        .remote_mods()
        .map(|(mod_ref, _)| {
            let borrowed = mod_ref.borrow(&thunderstore)?;
            Ok(borrowed.version.full_name.clone())
        })
        .collect::<Result<Vec<_>>>()
        .map(|deps| deps.join("\n"))
}
