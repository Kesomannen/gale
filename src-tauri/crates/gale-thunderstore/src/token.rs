use std::sync::LazyLock;

use eyre::Result;
use keyring::Entry;
use tracing::info;

static ENTRY: LazyLock<keyring::Result<Entry>> =
    LazyLock::new(|| Entry::new("thunderstore", "api_token"));

fn entry() -> Result<&'static keyring::Entry> {
    match &*ENTRY {
        Ok(entry) => Ok(entry),
        Err(err) => Err(err.into()),
    }
}

pub fn get() -> Result<Option<String>> {
    match entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn set(token: &str) -> Result<()> {
    info!("setting thunderstore token");
    entry()?.set_password(token)?;
    Ok(())
}

pub fn clear() -> Result<()> {
    info!("clearing thunderstore token");
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
