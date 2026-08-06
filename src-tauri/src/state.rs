use std::sync::{Arc, Mutex, MutexGuard};

use eyre::{Context, Result};
use http_cache_reqwest::{CACacheManager, CacheMode, HttpCache, HttpCacheOptions};
use serde::Serialize;
use tauri::{AppHandle, Manager, command};

use crate::{
    db::{self, Db},
    events::EventBuffer,
    prefs::Prefs,
    profile::{self, ModManager, install::queue::InstallQueue, sync},
    thunderstore::{self, Thunderstore},
};

pub struct AppState {
    http: reqwest_middleware::ClientWithMiddleware,
    prefs: Mutex<Prefs>,
    manager: Mutex<ModManager>,
    thunderstore: Mutex<Thunderstore>,
    db: Db,
    install_queue: Arc<InstallQueue>,
    sync_auth: sync::auth::State,
    sync_socket: sync::socket::State,
    event_buffer: EventBuffer,
    is_first_run: bool,
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
    let http = create_http_client().context("failed to init http client")?;

    let (db, db_existed) = db::init().context("failed to init database")?;

    let (data, mut prefs, creds, migrated) = db.read()?;

    prefs.init(&db, app).context("failed to init prefs")?;

    let manager = profile::setup(data, &prefs, &db, app).context("failed to init profiles")?;
    let thunderstore = Thunderstore::new();

    let state = AppState {
        db,
        http,
        prefs: Mutex::new(prefs),
        manager: Mutex::new(manager),
        thunderstore: Mutex::new(thunderstore),
        sync_auth: sync::auth::State::new(creds),
        sync_socket: sync::socket::State::new(app.to_owned()),
        install_queue: InstallQueue::new(app.to_owned()),
        event_buffer: EventBuffer::new(app.to_owned()),
        is_first_run: !db_existed && !migrated,
    };

    app.manage(state);

    thunderstore::start(app);

    let manager = app.lock_manager();
    manager.active_game().update_window_title(app).ok();
    app.sync_socket().subscribe(manager.active_profile());

    Ok(())
}

fn create_http_client() -> Result<reqwest_middleware::ClientWithMiddleware> {
    let base = reqwest::Client::builder()
        .user_agent(concat!("Kesomannen-Gale/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let cache_path = crate::util::path::default_app_cache_dir().join("http");

    let cache = http_cache_reqwest::Cache(HttpCache {
        mode: CacheMode::NoCache,
        manager: CACacheManager::new(cache_path, false),
        options: HttpCacheOptions::default(),
    });

    let http = reqwest_middleware::ClientBuilder::new(base)
        .with(cache)
        .build();

    Ok(http)
}

pub trait ManagerExt<R> {
    fn app_state(&self) -> &AppState;

    fn http(&self) -> &reqwest_middleware::ClientWithMiddleware {
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

    fn event_buffer(&self) -> &EventBuffer {
        &self.app_state().event_buffer
    }

    fn emit_buffered(&self, event: impl Into<String>, content: &impl Serialize) {
        self.event_buffer().emit(event, content);
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
