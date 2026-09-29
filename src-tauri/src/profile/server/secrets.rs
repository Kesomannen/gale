use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use eyre::{Context, Result};
use keyring::Entry;

use super::config::{RemoteAuthentication, RemoteProtocol, RemoteServerSettings};

const SERVICE: &str = "com.kesomannen.gale.dedicated-server";

static ENTRIES: LazyLock<Mutex<HashMap<(i64, ServerSecret), Entry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerSecret {
    GamePassword,
    SftpPassword,
    FtpPassword,
    SshKeyPassphrase,
}

impl ServerSecret {
    pub fn for_remote(settings: &RemoteServerSettings) -> Option<Self> {
        match (settings.protocol, settings.authentication) {
            (RemoteProtocol::Sftp, RemoteAuthentication::Password) => Some(Self::SftpPassword),
            (RemoteProtocol::Sftp, RemoteAuthentication::PrivateKey) => {
                Some(Self::SshKeyPassphrase)
            }
            (RemoteProtocol::Sftp, RemoteAuthentication::Agent) => None,
            _ => Some(Self::FtpPassword),
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Self::GamePassword => "game-password",
            Self::SftpPassword => "sftp-password",
            Self::FtpPassword => "ftp-password",
            Self::SshKeyPassphrase => "ssh-key-passphrase",
        }
    }

    fn with_entry<T>(self, profile_id: i64, action: impl FnOnce(&Entry) -> Result<T>) -> Result<T> {
        let mut entries = ENTRIES.lock().unwrap();
        let entry = match entries.entry((profile_id, self)) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(
                Entry::new(SERVICE, &format!("profile-{profile_id}-{}", self.suffix()))
                    .context("failed to access OS credential store")?,
            ),
        };

        action(entry)
    }

    pub fn set(self, profile_id: i64, value: &str) -> Result<()> {
        if value.is_empty() {
            return self.remove(profile_id);
        }

        self.with_entry(profile_id, |entry| {
            entry
                .set_password(value)
                .context("failed to save credential")
        })
    }

    pub fn get(self, profile_id: i64) -> Result<Option<String>> {
        self.with_entry(profile_id, |entry| match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err).context("failed to read credential"),
        })
    }

    pub fn remove(self, profile_id: i64) -> Result<()> {
        self.with_entry(profile_id, |entry| match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err).context("failed to remove credential"),
        })
    }

    pub fn resolve(self, profile_id: i64, value: &str) -> Result<String> {
        if value.is_empty() {
            Ok(self.get(profile_id)?.unwrap_or_default())
        } else {
            Ok(value.to_owned())
        }
    }
}
