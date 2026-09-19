use std::sync::Arc;

use eyre::{Context, OptionExt, ensure};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, command};
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    profile::{
        server::{
            args,
            deploy::{self, DeploymentPreviewResult, DeploymentProgress, DeploymentResult},
            local,
            remote::{self, ConnectionTestResult},
            runtime::{self, ServerStatus, SharedChild},
            secrets::{ServerSecret, ServerSecrets},
            settings::{ProfileServerSettings, RemoteServerSettings},
            spec::DeploymentSpec,
        },
        sync,
    },
    state::ManagerExt,
    util::cmd::Result,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchDedicatedServerRequest {
    /// Settings to launch with. Falls back to the profile's stored settings,
    /// so an already-configured server can launch immediately.
    #[serde(default)]
    pub settings: Option<ProfileServerSettings>,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub remember_password: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerRequest {
    pub settings: RemoteServerSettings,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub remember_password: bool,
}

#[command]
pub fn get_dedicated_server_settings(app: AppHandle) -> Option<ProfileServerSettings> {
    app.lock_manager().active_profile().server_settings.clone()
}

#[command]
pub fn set_dedicated_server_settings(
    settings: ProfileServerSettings,
    app: AppHandle,
) -> Result<()> {
    settings.validate()?;
    Ok(save_settings(&app, settings)?)
}

#[command]
pub async fn launch_dedicated_server(
    request: LaunchDedicatedServerRequest,
    app: AppHandle,
) -> Result<ServerStatus> {
    let profile_id = active_profile_id(&app);

    if app.lock_prefs().pull_before_launch {
        sync::pull_profile(false, profile_id, &app).await?;
    }

    ensure_no_pending_installs(&app)?;

    if app.lock_server_runtime().is_running() {
        return Err(eyre::eyre!("a Gale-managed dedicated server is already running").into());
    }

    let secrets = ServerSecrets::for_profile(profile_id)?;
    let password = secrets.resolve(ServerSecret::GamePassword, &request.password)?;

    let (game, stored_settings) = {
        let manager = app.lock_manager();
        (
            manager.active_game().game,
            manager.active_profile().server_settings.clone(),
        )
    };

    let settings = match request.settings {
        Some(settings) => {
            args::validate_game_args(game, &settings.local, &password)?;
            save_settings(&app, settings.clone())?;
            settings
        }
        None => stored_settings.ok_or_eyre("the dedicated server has not been configured yet")?,
    };

    args::validate_game_args(game, &settings.local, &password)?;
    secrets.persist(
        ServerSecret::GamePassword,
        &password,
        request.remember_password,
    )?;

    let process = {
        let prefs = app.lock_prefs();
        let manager = app.lock_manager();

        local::launch(
            manager.active_game(),
            manager.active_profile(),
            &settings.local,
            &password,
            &prefs,
        )?
    };

    let pid = process
        .child
        .id()
        .ok_or_eyre("dedicated server process has no pid")?;
    let child: SharedChild = Arc::new(Mutex::new(process.child));

    let status = {
        let mut runtime = app.lock_server_runtime();
        runtime.register(
            process.profile_id,
            process.game,
            process.server_dir,
            pid,
            child.clone(),
        )?
    };

    runtime::emit_status(&app, &status);
    runtime::watch(app.clone(), child, pid);

    Ok(status)
}

#[command]
pub async fn test_remote_server_connection(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<ConnectionTestResult> {
    let profile_id = active_profile_id(&app);
    request.settings.validate()?;

    let secrets = ServerSecrets::for_profile(profile_id)?;
    let credential = remote_credential(&secrets, &request.settings, &request.password)?;

    let settings = request.settings.clone();
    let password = credential.clone();

    let result = tokio::task::spawn_blocking(move || remote::test_connection(&settings, &password))
        .await
        .map_err(|err| eyre::eyre!("remote connection worker failed: {err}"))??;

    if matches!(result, ConnectionTestResult::Connected { .. }) {
        save_remote_request(&app, &secrets, &request, &credential)?;
    }

    Ok(result)
}

#[command]
pub async fn deploy_remote_server(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<DeploymentResult> {
    ensure_no_pending_installs(&app)?;
    request.settings.validate()?;

    let (profile_id, profile_dir, mod_loader) = active_profile_target(&app);
    let spec = DeploymentSpec::for_loader(mod_loader)?;

    let secrets = ServerSecrets::for_profile(profile_id)?;
    let credential = remote_credential(&secrets, &request.settings, &request.password)?;

    let settings = request.settings.clone();
    let password = credential.clone();
    let progress_app = app.clone();

    let result = tokio::task::spawn_blocking(move || {
        deploy::deploy(
            &profile_dir,
            &spec,
            &settings,
            &password,
            |progress: DeploymentProgress| {
                if let Err(err) = progress_app.emit("server_deployment_progress", progress) {
                    warn!(?err, "failed to emit deployment progress");
                }
            },
        )
    })
    .await
    .map_err(|err| eyre::eyre!("remote deployment worker failed: {err}"))??;

    if matches!(result, DeploymentResult::Deployed { .. }) {
        save_remote_request(&app, &secrets, &request, &credential)?;
    }

    Ok(result)
}

#[command]
pub async fn preview_remote_server_deployment(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<DeploymentPreviewResult> {
    ensure_no_pending_installs(&app)?;
    request.settings.validate()?;

    let (profile_id, profile_dir, mod_loader) = active_profile_target(&app);
    let spec = DeploymentSpec::for_loader(mod_loader)?;

    let secrets = ServerSecrets::for_profile(profile_id)?;
    let credential = remote_credential(&secrets, &request.settings, &request.password)?;

    let settings = request.settings.clone();
    let password = credential.clone();

    let result = tokio::task::spawn_blocking(move || {
        deploy::preview(&profile_dir, &spec, &settings, &password)
    })
    .await
    .map_err(|err| eyre::eyre!("deployment preview worker failed: {err}"))??;

    if matches!(result, DeploymentPreviewResult::Preview { .. }) {
        save_remote_request(&app, &secrets, &request, &credential)?;
    }

    Ok(result)
}

#[command]
pub fn get_dedicated_server_status(app: AppHandle) -> Result<ServerStatus> {
    Ok(app.lock_server_runtime().status())
}

#[command]
pub fn open_dedicated_server_dir(app: AppHandle) -> Result<()> {
    let running_dir = { app.lock_server_runtime().server_dir() };

    let path = match running_dir {
        Some(path) => path,

        None => {
            let prefs = app.lock_prefs();
            let manager = app.lock_manager();
            let game = manager.active_game();

            local::locate_server_dir(game, &prefs)?.0
        }
    };

    open::that(&path).wrap_err_with(|| {
        format!(
            "failed to open dedicated server directory {}",
            path.display()
        )
    })?;

    Ok(())
}

#[command]
pub async fn force_stop_dedicated_server(app: AppHandle) -> Result<()> {
    let child = {
        let mut runtime = app.lock_server_runtime();
        runtime.take()
    };

    match child {
        Some(child) => runtime::kill(app, child).await?,
        // Already stopped — still report the status so the UI stays in sync.
        None => runtime::emit_status(&app, &app.lock_server_runtime().status()),
    }

    Ok(())
}

/// The profile pinned for a remote operation, captured before any await so
/// nothing can silently switch underneath it.
fn active_profile_target(
    app: &AppHandle,
) -> (
    i64,
    std::path::PathBuf,
    &'static crate::game::mod_loader::ModLoader<'static>,
) {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    (
        profile.id,
        profile.path.clone(),
        &manager.active_game().game.mod_loader,
    )
}

fn ensure_no_pending_installs(app: &AppHandle) -> eyre::Result<()> {
    ensure!(
        !app.install_queue().lock().is_processing(),
        "please wait for mod installations to finish before launching the server"
    );

    Ok(())
}

fn active_profile_id(app: &AppHandle) -> i64 {
    app.lock_manager().active_profile().id
}

fn remote_credential(
    secrets: &ServerSecrets,
    settings: &RemoteServerSettings,
    provided: &str,
) -> eyre::Result<String> {
    match ServerSecret::required_by(settings) {
        Some(secret) => secrets.resolve(secret, provided),
        None => Ok(String::new()),
    }
}

fn save_remote_request(
    app: &AppHandle,
    secrets: &ServerSecrets,
    request: &RemoteServerRequest,
    credential: &str,
) -> eyre::Result<()> {
    save_remote_settings(app, request.settings.clone())?;

    if let Some(secret) = ServerSecret::required_by(&request.settings) {
        secrets.persist(secret, credential, request.remember_password)?;
    }

    Ok(())
}

fn save_settings(app: &AppHandle, settings: ProfileServerSettings) -> eyre::Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.server_settings = Some(settings);
    profile.save(app, true)
}

fn save_remote_settings(app: &AppHandle, remote: RemoteServerSettings) -> eyre::Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    let mut settings = profile.server_settings.clone().unwrap_or_default();
    settings.remote = remote;
    profile.server_settings = Some(settings);
    profile.save(app, true)
}
