use anyhow::Result;
use keyring::Entry;

lazy_static! {
    static ref ENTRY: keyring::Entry =
        Entry::new("thunderstore", "api_token").expect("failed to create keyring entry");
}

pub fn get() -> Result<Option<String>> {
    match ENTRY.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn set(token: &str) -> Result<()> {
    ENTRY.set_password(token)?;
    Ok(())
}

pub fn clear() -> Result<()> {
    match ENTRY.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
