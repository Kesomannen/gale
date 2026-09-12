use eyre::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "com.kesomannen.gale.dedicated-server";

#[derive(Debug, Clone, Copy)]
pub enum ServerSecret {
    GamePassword,
    SftpPassword,
    FtpPassword,
    SshKeyPassphrase,
}

impl ServerSecret {
    fn suffix(self) -> &'static str {
        match self {
            Self::GamePassword => "game-password",
            Self::SftpPassword => "sftp-password",
            Self::FtpPassword => "ftp-password",
            Self::SshKeyPassphrase => "ssh-key-passphrase",
        }
    }
}

fn entry(profile_id: i64, secret: ServerSecret) -> Result<Entry> {
    Entry::new(
        SERVICE,
        &format!("profile-{profile_id}-{}", secret.suffix()),
    )
    .context("failed to access OS credential store")
}

pub fn set(profile_id: i64, secret: ServerSecret, value: &str) -> Result<()> {
    let entry = entry(profile_id, secret)?;

    if value.is_empty() {
        return remove(profile_id, secret);
    }

    entry
        .set_password(value)
        .context("failed to save credential")
}

pub fn get(profile_id: i64, secret: ServerSecret) -> Result<Option<String>> {
    match entry(profile_id, secret)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err).context("failed to read credential"),
    }
}

pub fn remove(profile_id: i64, secret: ServerSecret) -> Result<()> {
    match entry(profile_id, secret)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("failed to remove credential"),
    }
}
