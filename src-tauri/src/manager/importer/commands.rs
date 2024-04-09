use std::path::PathBuf;

use crate::{
    command_util::{Result, StateMutex},
    manager::ModManager,
    prefs::Prefs,
    thunderstore::Thunderstore,
    NetworkClient,
};

use super::ModpackArgs;
use tauri::{AppHandle, State};
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
pub async fn import_code(key: Uuid, app: AppHandle) -> Result<()> {
    super::import_code(key, &app).await?;

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
pub async fn import_local_mod(
    path: PathBuf,
    app: AppHandle,
) -> Result<()> {
    super::import_local_mod(path, &app).await?;

    Ok(())
}
