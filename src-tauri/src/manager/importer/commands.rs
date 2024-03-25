use std::path::PathBuf;

use crate::{
    manager::{self, ModManager},
    thunderstore::ThunderstoreState,
    util,
};

use super::ModpackArgs;

type Result<T> = util::CommandResult<T>;

#[tauri::command]
pub fn export_pack(
    path: PathBuf,
    args: ModpackArgs,
    manager: tauri::State<ModManager>,
    thunderstore: tauri::State<ThunderstoreState>,
) -> Result<()> {
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = manager::get_active_profile(&mut profiles, &manager)?;

    let mod_map = thunderstore.all_mods.lock().unwrap();

    let zip_path = path.join(&args.name).with_extension("zip");
    profile.export_pack(&zip_path, args, &mod_map)?;

    let _ = open::that(&zip_path);
    Ok(())
}
