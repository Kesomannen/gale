use std::path::PathBuf;

use eyre::anyhow;
use tauri::{command, AppHandle};
use uuid::Uuid;

use crate::{
    profile::install::InstallOptions,
    thunderstore::{self},
    util::cmd::Result,
};

use super::{
    r2modman::{self},
    ImportData,
};

#[command]
pub async fn import_profile(data: ImportData, import_all: bool, app: AppHandle) -> Result<()> {
    super::import_profile(data, InstallOptions::default(), import_all, &app).await?;

    Ok(())
}

#[command]
pub async fn read_profile_code(key: &str, app: AppHandle) -> Result<ImportData> {
    let key = Uuid::parse_str(key).map_err(|_| anyhow!("invalid code format"))?;

    let data = super::read_code(key, &app).await?;

    Ok(data)
}

#[command]
pub async fn read_profile_file(path: PathBuf) -> Result<ImportData> {
    let data = super::import_file_from_path(path)?;

    Ok(data)
}

#[command]
pub async fn read_profile_base64(base64: String) -> Result<ImportData> {
    let data = super::read_base64(&base64)?;

    Ok(data)
}

#[command]
pub async fn import_local_mod(path: PathBuf, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    super::import_local_mod(
        path,
        None,
        &app,
        InstallOptions::default().can_cancel(false),
    )
    .await?;

    Ok(())
}

#[command]
pub async fn import_local_mod_base64(base64: String, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    super::import_local_mod_base64(base64, &app, InstallOptions::default().can_cancel(false))
        .await?;

    Ok(())
}

#[command]
pub fn get_r2modman_info(
    path: Option<PathBuf>,
    app: AppHandle,
) -> Result<Option<r2modman::ProfileImportData>> {
    let info = r2modman::gather_info(path, &app)?;

    Ok(info)
}

#[command]
pub async fn import_r2modman(path: PathBuf, include: Vec<bool>, app: AppHandle) -> Result<()> {
    r2modman::import(path, &include, &app).await?;

    Ok(())
}
