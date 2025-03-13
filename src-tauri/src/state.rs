use std::sync::{Mutex, MutexGuard};

use eyre::{Context, Result};
use tauri::{AppHandle, Manager};

use crate::{
    db::{self, Db},
    prefs::Prefs,
    profile::{self, ModManager},
    thunderstore::{self, Thunderstore},
};

pub struct AppState {
    http: reqwest::Client,
    prefs: Mutex<Prefs>,
    manager: Mutex<ModManager>,
    thunderstore: Mutex<Thunderstore>,
    db: Db,
}

impl AppState {
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn lock_prefs(&self) -> MutexGuard<'_, Prefs> {
        self.prefs.lock().unwrap()
    }

    pub fn lock_manager(&self) -> MutexGuard<'_, ModManager> {
        self.manager.lock().unwrap()
    }

    pub fn lock_thunderstore(&self) -> MutexGuard<'_, Thunderstore> {
        self.thunderstore.lock().unwrap()
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let http = reqwest::Client::builder()
        .user_agent("Kesomannen-gale")
        .build()
        .context("failed to init http client")?;

    let db = db::init().context("failed to init database")?;

    let prefs = Prefs::create(app).context("failed to init prefs")?;
    let manager = profile::setup(&prefs, &db, app).context("failed to init profiles")?;
    let thunderstore = Thunderstore::default();

    let state = AppState {
        db,
        http,
        prefs: Mutex::new(prefs),
        manager: Mutex::new(manager),
        thunderstore: Mutex::new(thunderstore),
    };

    app.manage(state);

    thunderstore::start(app);

    Ok(())
}

pub trait ManagerExt<R> {
    fn app_state(&self) -> &AppState;

    fn http(&self) -> &reqwest::Client {
        self.app_state().http()
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
        self.app_state().db()
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
