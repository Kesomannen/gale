use std::sync::LazyLock;

use eyre::Result;
use keyring::Entry;
use tracing::info;
use crate::thunderstore::Backend;

static THUNDERSTORE_ENTRY: LazyLock<keyring::Result<Entry>> =
    LazyLock::new(|| Entry::new("thunderstore", "api_token"));
static HEXIUM_ENTRY: LazyLock<keyring::Result<Entry>> =
    LazyLock::new(|| Entry::new("hexium", "api_token"));

fn entry(backend: Backend) -> Result<&'static keyring::Entry> {
    let entry = match backend {
        Backend::Thunderstore => &*THUNDERSTORE_ENTRY,
        Backend::Hexium => &*HEXIUM_ENTRY,
    };
    match entry {
        Ok(entry) => Ok(entry),
        Err(err) => Err(err.into()),
    }
}

pub fn get(backend: Backend) -> Result<Option<String>> {
    match entry(backend)?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn set(backend: Backend, token: &str) -> Result<()> {
    info!("setting {backend} token");
    entry(backend)?.set_password(token)?;
    Ok(())
}

pub fn clear(backend: Backend) -> Result<()> {
    info!("clearing {backend} token");
    match entry(backend)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
