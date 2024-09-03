use std::path::PathBuf;

pub const APP_GUID: &str = "com.kesomannen.gale";

pub fn app_config_dir() -> PathBuf {
    app_dir("config", dirs_next::config_dir())
}

pub fn app_data_dir() -> PathBuf {
    app_dir("data", dirs_next::data_dir())
}

pub fn app_cache_dir() -> PathBuf {
    app_dir("cache", dirs_next::cache_dir())
}

fn app_dir(id: &str, base: Option<PathBuf>) -> PathBuf {
    let mut path = base.unwrap_or_else(|| panic!("failed to resolve {} dir", id));
    path.push(APP_GUID);
    path
}
