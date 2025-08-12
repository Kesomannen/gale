use std::path::PathBuf;

use eyre::anyhow;
use serde::Serialize;
use tauri::{command, AppHandle};
use uuid::Uuid;

use crate::{
    profile::{import::ImportOptions, install::InstallOptions},
    state::ManagerExt,
    thunderstore::{self, VersionIdent},
    util::cmd::Result,
};

use super::{
    r2modman::{self},
    ImportData,
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontendImportData {
    #[serde(flatten)]
    inner: ImportData,
    missing_mods: Vec<VersionIdent>,
}

impl FrontendImportData {
    pub fn new(mut inner: ImportData, app: &AppHandle) -> Self {
        let thunderstore = app.lock_thunderstore();
        let mut missing_mods = Vec::new();

        inner
            .manifest
            .mods
            .retain(|r2_mod| match r2_mod.into_install(&thunderstore) {
                Ok(_) => true,
                Err(_) => {
                    missing_mods.push(r2_mod.version_ident());
                    false
                }
            });

        Self {
            inner,
            missing_mods,
        }
    }
}

#[command]
pub async fn import_profile(data: ImportData, import_all: bool, app: AppHandle) -> Result<()> {
    super::import_profile(
        data,
        ImportOptions::default().import_all(import_all),
        InstallOptions::default(),
        &app,
    )
    .await?;

    Ok(())
}

#[command]
pub async fn read_profile_code(key: &str, app: AppHandle) -> Result<FrontendImportData> {
    let key = Uuid::parse_str(key).map_err(|_| anyhow!("invalid code format"))?;

    let data = super::read_code(key, &app).await?;

    Ok(FrontendImportData::new(data, &app))
}

#[command]
pub async fn read_profile_file(path: PathBuf, app: AppHandle) -> Result<FrontendImportData> {
    let data = super::read_file_from(path)?;

    Ok(FrontendImportData::new(data, &app))
}

#[command]
pub async fn read_profile_base64(base64: String, app: AppHandle) -> Result<FrontendImportData> {
    let data = super::read_base64(&base64)?;

    Ok(FrontendImportData::new(data, &app))
}

#[command]
pub async fn import_local_mod(path: PathBuf, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    super::import_local_mod(path, None, &app, InstallOptions::default()).await?;

    Ok(())
}

#[command]
pub async fn import_local_mod_base64(base64: String, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    super::import_local_mod_base64(base64, &app, InstallOptions::default()).await?;

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
