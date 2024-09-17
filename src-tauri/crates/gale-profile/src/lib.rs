use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use install::InstallQueue;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::types::Uuid;
use sqlx::Row;
use std::borrow::Cow;
use std::path::PathBuf;
use tauri::{
    async_runtime, generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Emitter, Manager,
};

mod actions;
mod commands;
mod get;
pub mod install;
mod launch_;

pub use actions::*;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-profile")
        .setup(|app, _| {
            let queue = InstallQueue::new();
            app.manage(queue);

            let handle = app.app_handle().to_owned();
            async_runtime::spawn(install::handler(handle));

            let handle = app.app_handle().to_owned();
            async_runtime::spawn(async move {
                if let Err(err) = create_default_profile(handle).await {
                    log::error!("failed to create default profile: {:#}", err);
                }
            });

            Ok(())
        })
        .invoke_handler(generate_handler![
            commands::create,
            commands::delete,
            commands::rename,
            commands::get,
            commands::force_uninstall_mod,
            commands::force_toggle_mod,
            commands::queue_install,
            commands::launch
        ])
        .build()
}

async fn create_default_profile(app: AppHandle) -> Result<()> {
    let state = app.app_state();

    let profiles: u32 = sqlx::query("SELECT COUNT(*) FROM profiles")
        .fetch_one(&state.db)
        .await?
        .get(0);

    if profiles > 0 {
        return Ok(());
    }

    let path = PathBuf::from(r"D:\Gale\v2\profiles\Default");
    let id = actions::create("Default", &path, 2, &state).await?;

    log::info!("created default profile with id {}", id);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ProfileModSource {
    Thunderstore {
        identifier: VersionId,
        version_uuid: Uuid,
    },
    Github {
        owner: String,
        repo: String,
        tag: String,
    },
    Local {
        full_name: String,
        version: String,
    },
}

impl ProfileModSource {
    /// Unique identifier for the mod, excluding the version.
    fn full_name(&self) -> Cow<'_, str> {
        match self {
            ProfileModSource::Thunderstore { identifier, .. } => {
                Cow::Borrowed(identifier.full_name())
            }
            ProfileModSource::Github { owner, repo, .. } => {
                Cow::Owned(format!("{}-{}", owner, repo))
            }
            ProfileModSource::Local { full_name, .. } => Cow::Borrowed(full_name),
        }
    }
}

async fn emit_update(id: i64, state: &AppState, app: &AppHandle) -> Result<()> {
    let info = crate::get::single(id, state).await?;
    app.emit("profile-update", info)?;
    Ok(())
}

async fn scan_mod(profile_mod_id: i64, state: &AppState) -> Result<impl Iterator<Item = PathBuf>> {
    let (source, mut path) = sqlx::query!(
        r#"SELECT
            pm.source AS "source: Json<ProfileModSource>",
            p.path
        FROM
            profile_mods pm
            JOIN profiles p
                ON pm.profile_id = p.id
        WHERE pm.id = ?"#,
        profile_mod_id
    )
    .map(|record| (record.source.0, PathBuf::from(record.path)))
    .fetch_one(&state.db)
    .await?;

    let full_name = source.full_name().into_owned();

    path.push("BepInEx");

    Ok(["plugins", "patchers", "monomod", "core"]
        .iter()
        .map(move |dir| path.join(dir).join(&full_name))
        .filter(|path| path.exists()))
}
