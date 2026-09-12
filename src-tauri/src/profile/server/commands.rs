use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use eyre::{Context, ensure};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, command};
use tracing::{info, warn};

use crate::{
    profile::{
        server::{
            config::{
                DedicatedServerSettings, RemoteAuthentication, RemoteProtocol, RemoteServerSettings,
            },
            deploy::{self, DeploymentPreviewResult, DeploymentProgress, DeploymentResult},
            local,
            remote::{self, ConnectionTestResult},
            runtime::{ServerStatus, SharedChild},
            secrets::{self, ServerSecret},
        },
        sync,
    },
    state::ManagerExt,
    util::cmd::Result,
};

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchDedicatedServerRequest {
    pub settings: DedicatedServerSettings,
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
pub fn get_dedicated_server_settings(app: AppHandle) -> DedicatedServerSettings {
    app.lock_manager().active_profile().server_settings.clone()
}

#[command]
pub fn set_dedicated_server_settings(
    settings: DedicatedServerSettings,
    app: AppHandle,
) -> Result<()> {
    save_settings(&app, settings)?;
    Ok(())
}

#[command]
pub async fn launch_dedicated_server(
    request: LaunchDedicatedServerRequest,
    app: AppHandle,
) -> Result<ServerStatus> {
    if app.lock_prefs().pull_before_launch {
        sync::pull_profile(false, &app).await?;
    }

    ensure_no_pending_installs(&app)?;

    {
        let runtime = app.lock_server_runtime();

        if runtime.is_running() {
            return Err(eyre::eyre!("a Gale-managed dedicated server is already running").into());
        }
    }

    let profile_id = active_profile_id(&app);
    let password = password_for_request(profile_id, ServerSecret::GamePassword, &request.password)?;
    request.settings.validate_local(&password)?;
    save_settings(&app, request.settings.clone())?;
    persist_secret(
        profile_id,
        ServerSecret::GamePassword,
        &password,
        request.remember_password,
    )?;

    let process = {
        let prefs = app.lock_prefs();
        let manager = app.lock_manager();

        local::launch(manager.active_game(), &request.settings, &password, &prefs)?
    };

    let child: SharedChild = Arc::new(Mutex::new(process.child));

    let pid = child
        .lock()
        .expect("server child process mutex poisoned")
        .id();

    let status = {
        let mut runtime = app.lock_server_runtime();

        runtime.register(
            process.profile_id,
            process.game_slug,
            process.server_dir,
            child.clone(),
        )?
    };

    emit_status(&app, &status);

    notify_profile_changed(&app, process.profile_id);

    spawn_process_watcher(app, child, pid);

    Ok(status)
}

#[command]
pub async fn test_remote_server_connection(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<ConnectionTestResult> {
    let profile_id = active_profile_id(&app);
    let password = remote_credential(profile_id, &request.settings, &request.password)?;
    let settings_for_thread = request.settings.clone();
    let password_for_thread = password.clone();

    let result = tokio::task::spawn_blocking(move || {
        remote::test_connection(&settings_for_thread, &password_for_thread)
    })
    .await
    .map_err(|err| eyre::eyre!("remote connection worker failed: {err}"))??;

    if matches!(result, ConnectionTestResult::Connected { .. }) {
        save_remote_request(&app, profile_id, &request, &password)?;
    }

    Ok(result)
}

#[command]
pub async fn deploy_remote_server(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<DeploymentResult> {
    ensure_no_pending_installs(&app)?;

    let (profile_id, profile_dir, profile_mods) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();
        (profile.id, profile.path.clone(), profile.mods.clone())
    };
    let client_only_mods = client_only_mods(&profile_mods, &app.lock_thunderstore());
    let password = remote_credential(profile_id, &request.settings, &request.password)?;
    let settings_for_thread = request.settings.clone();
    let password_for_thread = password.clone();
    let progress_app = app.clone();

    let result = tokio::task::spawn_blocking(move || {
        deploy::deploy(
            &profile_dir,
            client_only_mods,
            &settings_for_thread,
            &password_for_thread,
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
        save_remote_request(&app, profile_id, &request, &password)?;
    }

    Ok(result)
}

#[command]
pub async fn preview_remote_server_deployment(
    request: RemoteServerRequest,
    app: AppHandle,
) -> Result<DeploymentPreviewResult> {
    ensure_no_pending_installs(&app)?;
    let (profile_id, profile_dir, profile_mods) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();
        (profile.id, profile.path.clone(), profile.mods.clone())
    };
    let client_only_mods = client_only_mods(&profile_mods, &app.lock_thunderstore());
    let password = remote_credential(profile_id, &request.settings, &request.password)?;
    let settings_for_thread = request.settings.clone();
    let password_for_thread = password.clone();
    let result = tokio::task::spawn_blocking(move || {
        deploy::preview(
            &profile_dir,
            client_only_mods,
            &settings_for_thread,
            &password_for_thread,
        )
    })
    .await
    .map_err(|err| eyre::eyre!("deployment preview worker failed: {err}"))??;

    if matches!(result, DeploymentPreviewResult::Preview { .. }) {
        save_remote_request(&app, profile_id, &request, &password)?;
    }

    Ok(result)
}

fn client_only_mods(
    profile_mods: &[crate::profile::ProfileMod],
    thunderstore: &crate::thunderstore::Thunderstore,
) -> BTreeSet<String> {
    profile_mods
        .iter()
        .filter(|profile_mod| profile_mod.enabled)
        .filter_map(|profile_mod| {
            let remote = profile_mod.kind.as_thunderstore()?;
            let package = remote.id.borrow(thunderstore).ok()?.package;
            let client = package
                .categories
                .iter()
                .any(|category| category.eq_ignore_ascii_case("Client-side"));
            let server = package
                .categories
                .iter()
                .any(|category| category.eq_ignore_ascii_case("Server-side"));
            (client && !server).then(|| profile_mod.full_name().into_owned())
        })
        .collect()
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
pub fn force_stop_dedicated_server(app: AppHandle) -> Result<()> {
    let profile_id = {
        let mut runtime = app.lock_server_runtime();
        runtime.force_stop()?
    };

    if let Some(profile_id) = profile_id {
        notify_profile_changed(&app, profile_id);
    }

    let status = app.lock_server_runtime().status();

    emit_status(&app, &status);

    Ok(())
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

fn password_for_request(
    profile_id: i64,
    secret: ServerSecret,
    password: &str,
) -> eyre::Result<String> {
    if password.is_empty() {
        Ok(secrets::get(profile_id, secret)?.unwrap_or_default())
    } else {
        Ok(password.to_owned())
    }
}

fn remote_credential(
    profile_id: i64,
    settings: &RemoteServerSettings,
    credential: &str,
) -> eyre::Result<String> {
    let Some(secret) = remote_secret(settings.protocol, settings.authentication) else {
        return Ok(String::new());
    };

    password_for_request(profile_id, secret, credential)
}

fn persist_remote_credential(
    profile_id: i64,
    protocol: RemoteProtocol,
    authentication: RemoteAuthentication,
    credential: &str,
    remember: bool,
) -> eyre::Result<()> {
    let Some(secret) = remote_secret(protocol, authentication) else {
        return Ok(());
    };

    persist_secret(profile_id, secret, credential, remember)
}

fn save_remote_request(
    app: &AppHandle,
    profile_id: i64,
    request: &RemoteServerRequest,
    credential: &str,
) -> eyre::Result<()> {
    save_remote_settings(app, request.settings.clone())?;
    persist_remote_credential(
        profile_id,
        request.settings.protocol,
        request.settings.authentication,
        credential,
        request.remember_password,
    )
}

fn remote_secret(
    protocol: RemoteProtocol,
    authentication: RemoteAuthentication,
) -> Option<ServerSecret> {
    match (protocol, authentication) {
        (RemoteProtocol::Sftp, RemoteAuthentication::Password) => Some(ServerSecret::SftpPassword),
        (RemoteProtocol::Sftp, RemoteAuthentication::PrivateKey) => {
            Some(ServerSecret::SshKeyPassphrase)
        }
        (RemoteProtocol::Sftp, RemoteAuthentication::Agent) => None,
        _ => Some(ServerSecret::FtpPassword),
    }
}

fn persist_secret(
    profile_id: i64,
    secret: ServerSecret,
    password: &str,
    remember: bool,
) -> eyre::Result<()> {
    if remember {
        secrets::set(profile_id, secret, password)
    } else {
        secrets::remove(profile_id, secret)
    }
}

fn save_settings(app: &AppHandle, settings: DedicatedServerSettings) -> eyre::Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.server_settings = settings;
    profile.save(app, true)
}

fn save_remote_settings(app: &AppHandle, settings: RemoteServerSettings) -> eyre::Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.server_settings.remote = settings;
    profile.save(app, true)
}

fn spawn_process_watcher(app: AppHandle, child: SharedChild, pid: u32) {
    thread::spawn(move || {
        loop {
            let result = {
                let mut child = match child.lock() {
                    Ok(child) => child,

                    Err(err) => {
                        warn!(?err, "dedicated server process mutex was poisoned");

                        break;
                    }
                };

                child.try_wait()
            };

            match result {
                Ok(Some(exit_status)) => {
                    info!(pid, ?exit_status, "dedicated server exited");

                    let profile_id = {
                        let mut runtime = app.lock_server_runtime();

                        runtime.clear_if_pid(pid)
                    };

                    if let Some(profile_id) = profile_id {
                        notify_profile_changed(&app, profile_id);

                        let status = app.lock_server_runtime().status();

                        emit_status(&app, &status);
                    }

                    break;
                }

                Ok(None) => {
                    thread::sleep(PROCESS_POLL_INTERVAL);
                }

                Err(err) => {
                    warn!(pid, ?err, "failed to query dedicated server process");

                    let profile_id = {
                        let mut runtime = app.lock_server_runtime();

                        runtime.clear_if_pid(pid)
                    };

                    if let Some(profile_id) = profile_id {
                        notify_profile_changed(&app, profile_id);
                    }

                    let status = app.lock_server_runtime().status();

                    emit_status(&app, &status);

                    break;
                }
            }
        }
    });
}

fn emit_status(app: &AppHandle, status: &ServerStatus) {
    if let Err(err) = app.emit("server_status_changed", status) {
        warn!(?err, "failed to emit dedicated server status");
    }
}

fn notify_profile_changed(app: &AppHandle, profile_id: i64) {
    let manager = app.lock_manager();

    let Ok((_, profile)) = manager.profile_by_id(profile_id) else {
        return;
    };

    if let Err(err) = profile.notify_frontend(app) {
        warn!(
            ?err,
            profile_id, "failed to notify frontend about profile change"
        );
    }
}
