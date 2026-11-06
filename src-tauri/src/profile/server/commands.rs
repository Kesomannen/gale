use tauri::{AppHandle, Emitter, command};
use tracing::warn;

use crate::{
    profile::{
        server::{
            config::{self, ProfileServerSettings, RemoteServerSettings, ServerLocation},
            deploy::{
                self, DeploymentPreviewResult, DeploymentProgress, DeploymentResult,
                DeploymentSelection,
            },
            ensure_no_pending_installs, local,
            remote::{self, ConnectionTestResult},
            runtime::{self, ServerStatus},
            secrets::ServerSecret,
        },
        sync,
    },
    state::ManagerExt,
    util::cmd::Result,
};

#[command]
pub fn get_dedicated_server_settings(app: AppHandle) -> Option<ProfileServerSettings> {
    config::get(&app)
}

#[command]
pub fn set_dedicated_server_settings(
    settings: ProfileServerSettings,
    game_password: Option<String>,
    remote_credential: Option<String>,
    app: AppHandle,
) -> Result<()> {
    let profile_id = app.lock_manager().active_profile().id;
    if settings.location == ServerLocation::Local || game_password.is_some() {
        let password = match game_password.as_deref() {
            Some(password) => Some(password.to_owned()),
            None => ServerSecret::GamePassword.get(profile_id)?,
        };
        settings.local.validate(password.as_deref())?;
    }
    if settings.location == ServerLocation::Remote {
        settings.remote.validate()?;
        let credential = match ServerSecret::for_remote(&settings.remote) {
            Some(secret) => match remote_credential.as_deref() {
                Some(credential) => Some(credential.to_owned()),
                None => secret.get(profile_id)?,
            },
            None => None,
        };
        settings.remote.validate_credential(credential.as_deref())?;
    }
    if let Some(password) = &game_password {
        ServerSecret::GamePassword.set(profile_id, password)?;
    }
    if let Some(credential) = &remote_credential
        && let Some(secret) = ServerSecret::for_remote(&settings.remote)
    {
        secret.set(profile_id, credential)?;
    }

    config::save_for_profile(&app, profile_id, settings)?;

    Ok(())
}

#[command]
pub async fn launch_dedicated_server(
    settings: ProfileServerSettings,
    app: AppHandle,
) -> Result<ServerStatus> {
    if app.lock_server_runtime().is_running() {
        return Err(eyre::eyre!("a Gale-managed dedicated server is already running").into());
    }

    if app.lock_prefs().pull_before_launch {
        sync::pull_profile(false, &app).await?;
    }

    ensure_no_pending_installs(&app)?;

    let profile_id = app.lock_manager().active_profile().id;
    let password = ServerSecret::GamePassword.get(profile_id)?;

    let process = {
        let prefs = app.lock_prefs();
        let manager = app.lock_manager();

        local::launch(manager.active_game(), &settings.local, password, &prefs)?
    };

    Ok(runtime::start(app, process)?)
}

#[command]
pub async fn test_remote_server_connection(
    settings: ProfileServerSettings,
    app: AppHandle,
) -> Result<ConnectionTestResult> {
    let profile_id = app.lock_manager().active_profile().id;
    let password = remote_credential(profile_id, &settings.remote)?;
    let remote_settings = settings.remote.clone();

    let result = remote::run_blocking("remote connection", move || {
        remote::test_connection(&remote_settings, password.as_deref().unwrap_or_default())
    })
    .await?;

    if matches!(result, ConnectionTestResult::Connected { .. }) {
        config::save_for_profile(&app, profile_id, settings)?;
    }

    Ok(result)
}

#[command]
pub async fn deploy_remote_server(
    settings: ProfileServerSettings,
    selection: DeploymentSelection,
    app: AppHandle,
) -> Result<DeploymentResult> {
    ensure_no_pending_installs(&app)?;

    let (profile_id, profile_dir, game, client_only_mods) = deploy::active_profile(&app);
    let password = remote_credential(profile_id, &settings.remote)?;
    let remote_settings = settings.remote.clone();
    let progress_app = app.clone();

    let result = remote::run_blocking("remote deployment", move || {
        deploy::deploy(
            &profile_dir,
            game,
            client_only_mods,
            &remote_settings,
            password.as_deref().unwrap_or_default(),
            &selection,
            |progress: DeploymentProgress| {
                if let Err(err) = progress_app.emit("server_deployment_progress", progress) {
                    warn!(?err, "failed to emit deployment progress");
                }
            },
        )
    })
    .await?;

    if matches!(result, DeploymentResult::Deployed { .. }) {
        config::save_for_profile(&app, profile_id, settings)?;
    }

    Ok(result)
}

#[command]
pub async fn preview_remote_server_deployment(
    settings: ProfileServerSettings,
    app: AppHandle,
) -> Result<DeploymentPreviewResult> {
    ensure_no_pending_installs(&app)?;

    let (profile_id, profile_dir, game, client_only_mods) = deploy::active_profile(&app);
    let password = remote_credential(profile_id, &settings.remote)?;
    let remote_settings = settings.remote.clone();

    let result = remote::run_blocking("deployment preview", move || {
        deploy::preview(
            &profile_dir,
            game,
            client_only_mods,
            &remote_settings,
            password.as_deref().unwrap_or_default(),
        )
    })
    .await?;

    if matches!(result, DeploymentPreviewResult::Preview { .. }) {
        config::save_for_profile(&app, profile_id, settings)?;
    }

    Ok(result)
}

#[command]
pub fn get_dedicated_server_status(app: AppHandle) -> Result<ServerStatus> {
    Ok(app.lock_server_runtime().status())
}

fn remote_credential(
    profile_id: i64,
    settings: &RemoteServerSettings,
) -> eyre::Result<Option<String>> {
    let Some(secret) = ServerSecret::for_remote(settings) else {
        return Ok(None);
    };

    secret.get(profile_id)
}
