use std::path::PathBuf;

use eyre::anyhow;
use tauri::AppHandle;
use uuid::Uuid;

use crate::{profile::install::InstallOptions, util::cmd::Result};

use super::{
    r2modman::{self, ProfileImportData},
    ImportData,
};

#[tauri::command]
pub async fn import_data(data: ImportData, import_all: bool, app: AppHandle) -> Result<()> {
    super::import_data(data, InstallOptions::default(), import_all, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn import_code(key: &str, app: AppHandle) -> Result<ImportData> {
    let key = Uuid::parse_str(key).map_err(|_| anyhow!("invalid code format"))?;
    let data = super::import_code(key, &app).await?;

    Ok(data)
}

#[tauri::command]
pub fn import_file(path: PathBuf, app: AppHandle) -> Result<ImportData> {
    let data = super::import_file_from_path(path, &app)?;

    Ok(data)
}

#[tauri::command]
pub async fn import_local_mod(path: PathBuf, app: AppHandle) -> Result<()> {
    super::import_local_mod(path, &app, InstallOptions::default().can_cancel(false)).await?;

    Ok(())
}

#[tauri::command]
pub fn get_r2modman_info(
    path: Option<PathBuf>,
    app: AppHandle,
) -> Result<Option<ProfileImportData>> {
    let info = r2modman::gather_info(path, &app)?;

    Ok(info)
}

#[tauri::command]
pub async fn import_r2modman(path: PathBuf, include: Vec<bool>, app: AppHandle) -> Result<()> {
    r2modman::import(path, &include, &app).await?;

    Ok(())
}
