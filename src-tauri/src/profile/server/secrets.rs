use eyre::{Context, Result};
use keyring::Entry;

use super::settings::RemoteServerSettings;

const SERVICE: &str = "com.kesomannen.gale.dedicated-server";

#[derive(Debug, Clone, Copy)]
pub enum ServerSecret {
    GamePassword,
    SftpPassword,
    FtpPassword,
    SshKeyPassphrase,
}

/// Access to a profile's server credentials in the OS credential store.
/// The keyring entries are initialized once per server operation and reused
/// for every credential it touches.
pub struct ServerSecrets {
    game_password: Entry,
    sftp_password: Entry,
    ftp_password: Entry,
    ssh_key_passphrase: Entry,
}

impl ServerSecret {
    /// The credential a remote connection requires, if any. SSH agent
    /// authentication needs no stored credential.
    pub fn required_by(settings: &RemoteServerSettings) -> Option<Self> {
        use super::settings::{RemoteAuthentication, RemoteProtocol};

        match (settings.protocol, settings.authentication) {
            (RemoteProtocol::Sftp, RemoteAuthentication::Password) => Some(Self::SftpPassword),
            (RemoteProtocol::Sftp, RemoteAuthentication::PrivateKey) => {
                Some(Self::SshKeyPassphrase)
            }
            (RemoteProtocol::Sftp, RemoteAuthentication::Agent) => None,
            (_, _) => Some(Self::FtpPassword),
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

    fn entry(self, profile_id: i64) -> Result<Entry> {
        Entry::new(SERVICE, &format!("profile-{profile_id}-{}", self.suffix()))
            .context("failed to access OS credential store")
    }
}

impl ServerSecrets {
    pub fn for_profile(profile_id: i64) -> Result<Self> {
        Ok(Self {
            game_password: ServerSecret::GamePassword.entry(profile_id)?,
            sftp_password: ServerSecret::SftpPassword.entry(profile_id)?,
            ftp_password: ServerSecret::FtpPassword.entry(profile_id)?,
            ssh_key_passphrase: ServerSecret::SshKeyPassphrase.entry(profile_id)?,
        })
    }

    fn entry(&self, secret: ServerSecret) -> &Entry {
        match secret {
            ServerSecret::GamePassword => &self.game_password,
            ServerSecret::SftpPassword => &self.sftp_password,
            ServerSecret::FtpPassword => &self.ftp_password,
            ServerSecret::SshKeyPassphrase => &self.ssh_key_passphrase,
        }
    }

    /// Returns `provided` when given, otherwise the stored credential.
    pub fn resolve(&self, secret: ServerSecret, provided: &str) -> Result<String> {
        if !provided.is_empty() {
            return Ok(provided.to_owned());
        }

        self.get(secret).map(|value| value.unwrap_or_default())
    }

    pub fn get(&self, secret: ServerSecret) -> Result<Option<String>> {
        match self.entry(secret).get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err).context("failed to read credential"),
        }
    }

    /// Stores `value` when `remember` is set, clears it otherwise.
    pub fn persist(&self, secret: ServerSecret, value: &str, remember: bool) -> Result<()> {
        if remember && !value.is_empty() {
            self.set(secret, value)
        } else {
            self.remove(secret)
        }
    }

    pub fn set(&self, secret: ServerSecret, value: &str) -> Result<()> {
        self.entry(secret)
            .set_password(value)
            .context("failed to save credential")
    }

    pub fn remove(&self, secret: ServerSecret) -> Result<()> {
        match self.entry(secret).delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err).context("failed to remove credential"),
        }
    }
}
