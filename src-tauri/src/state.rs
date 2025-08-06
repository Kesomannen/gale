use std::sync::{Mutex, MutexGuard};

use eyre::{Context, Result};
use tauri::{command, AppHandle, Manager};

use crate::{
    db::{self, Db},
    prefs::Prefs,
    profile::{self, install::queue::InstallQueue, sync, ModManager},
    thunderstore::{self, Thunderstore},
};

pub struct AppState {
    pub http: reqwest::Client,
    pub prefs: Mutex<Prefs>,
    pub manager: Mutex<ModManager>,
    pub thunderstore: Mutex<Thunderstore>,
    pub db: Db,
    pub install_queue: InstallQueue,
    pub sync_auth: sync::auth::State,
    pub sync_socket: sync::socket::State,
    pub is_first_run: bool,
}

impl AppState {
    pub fn lock_prefs(&self) -> MutexGuard<'_, Prefs> {
        self.prefs.lock().unwrap()
    }

    pub fn lock_manager(&self) -> MutexGuard<'_, ModManager> {
        self.manager.lock().unwrap()
    }

    pub fn lock_thunderstore(&self) -> MutexGuard<'_, Thunderstore> {
        self.thunderstore.lock().unwrap()
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let http = reqwest::Client::builder()
        .user_agent(concat!("Kesomannen-Gale/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to init http client")?;

    let (db, db_existed) = db::init().context("failed to init database")?;

    let (data, mut prefs, creds, migrated) = db.read()?;

    prefs.init(&db, app).context("failed to init prefs")?;

    let manager = profile::setup(data, &prefs, &db, app).context("failed to init profiles")?;
    let thunderstore = Thunderstore::default();

    let state = AppState {
        db,
        http,
        prefs: Mutex::new(prefs),
        manager: Mutex::new(manager),
        thunderstore: Mutex::new(thunderstore),
        sync_auth: sync::auth::State::new(creds),
        sync_socket: sync::socket::State::new(app.to_owned()),
        install_queue: InstallQueue::new(app.to_owned()),
        is_first_run: !db_existed && !migrated,
    };

    app.manage(state);

    thunderstore::start(app);

    let manager = app.lock_manager();
    manager.active_game().update_window_title(app).ok();
    app.sync_socket().subscribe(manager.active_profile());

    Ok(())
}

pub trait ManagerExt<R> {
    fn app_state(&self) -> &AppState;

    fn http(&self) -> &reqwest::Client {
        &self.app_state().http
    }

    fn lock_prefs(&self) -> MutexGuard<'_, Prefs> {
        self.app_state().lock_prefs()
    }

    fn lock_manager(&self) -> MutexGuard<'_, ModManager> {
        self.app_state().lock_manager()
    }

    fn lock_thunderstore(&self) -> MutexGuard<'_, Thunderstore> {
        self.app_state().lock_thunderstore()
    }

    fn db(&self) -> &Db {
        &self.app_state().db
    }

    fn install_queue(&self) -> &InstallQueue {
        &self.app_state().install_queue
    }

    fn sync_auth(&self) -> &sync::auth::State {
        &self.app_state().sync_auth
    }

    fn sync_socket(&self) -> &sync::socket::State {
        &self.app_state().sync_socket
    }
}

impl<T, R> ManagerExt<R> for T
where
    T: tauri::Manager<R>,
    R: tauri::Runtime,
{
    fn app_state(&self) -> &AppState {
        self.state::<AppState>().inner()
    }
}

#[command]
pub fn is_first_run(app: AppHandle) -> bool {
    app.app_state().is_first_run
}
