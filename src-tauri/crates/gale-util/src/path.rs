use std::path::PathBuf;

pub const APP_GUID: &str = "com.kesomannen.gale";

pub fn default_app_config_dir() -> PathBuf {
    app_dir("config", dirs_next::config_dir())
}

pub fn default_app_data_dir() -> PathBuf {
    app_dir("data", dirs_next::data_dir())
}

fn app_dir(id: &str, base: Option<PathBuf>) -> PathBuf {
    let mut path = base.unwrap_or_else(|| panic!("failed to resolve {id} dir"));
    path.push(APP_GUID);
    path
}
