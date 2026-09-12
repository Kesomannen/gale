use eyre::{Result, ensure};
use serde::{Deserialize, Serialize};

const DEFAULT_SERVER_PORT: u16 = 2456;
const DEFAULT_SFTP_PORT: u16 = 22;
const MIN_PASSWORD_LENGTH: usize = 5;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerLocation {
    #[default]
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteAuthentication {
    #[default]
    Password,
    PrivateKey,
    Agent,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteProtocol {
    #[default]
    Sftp,
    Ftp,
    Ftps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DedicatedServerSettings {
    pub location: ServerLocation,
    pub server_name: String,
    pub world: String,
    pub port: u16,
    pub public_server: bool,
    pub crossplay: bool,
    pub extra_args: String,
    pub remote: RemoteServerSettings,
}

impl Default for DedicatedServerSettings {
    fn default() -> Self {
        Self {
            location: ServerLocation::Local,
            server_name: "My Valheim Server".to_owned(),
            world: "Dedicated".to_owned(),
            port: DEFAULT_SERVER_PORT,
            public_server: true,
            crossplay: false,
            extra_args: String::new(),
            remote: RemoteServerSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RemoteServerSettings {
    pub protocol: RemoteProtocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub server_directory: String,
    pub authentication: RemoteAuthentication,
    pub private_key_path: String,
    pub trusted_host_key: Option<String>,
    pub trusted_invalid_certificate_host: Option<String>,
}

impl Default for RemoteServerSettings {
    fn default() -> Self {
        Self {
            protocol: RemoteProtocol::Sftp,
            host: String::new(),
            port: DEFAULT_SFTP_PORT,
            username: String::new(),
            server_directory: String::new(),
            authentication: RemoteAuthentication::Password,
            private_key_path: String::new(),
            trusted_host_key: None,
            trusted_invalid_certificate_host: None,
        }
    }
}

impl DedicatedServerSettings {
    pub fn validate_local(&self, password: &str) -> Result<()> {
        ensure!(
            !self.server_name.trim().is_empty(),
            "server name cannot be empty"
        );

        ensure!(!self.world.trim().is_empty(), "world name cannot be empty");
        ensure!(self.port != 0, "server port cannot be 0");

        if !password.is_empty() {
            ensure!(
                password.chars().count() >= MIN_PASSWORD_LENGTH,
                "server password must contain at least {MIN_PASSWORD_LENGTH} characters"
            );

            let server_name = self.server_name.to_lowercase();
            let password = password.to_lowercase();

            ensure!(
                !server_name.contains(&password),
                "server password cannot be contained in the server name"
            );
        }

        Ok(())
    }
}

impl RemoteServerSettings {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.host.trim().is_empty(), "remote host cannot be empty");
        ensure!(self.port != 0, "remote port cannot be 0");
        ensure!(
            !self.username.trim().is_empty(),
            "remote username cannot be empty"
        );
        ensure!(
            !self.server_directory.trim().is_empty(),
            "remote server directory cannot be empty"
        );
        if self.protocol == RemoteProtocol::Sftp
            && self.authentication == RemoteAuthentication::PrivateKey
        {
            ensure!(
                !self.private_key_path.trim().is_empty(),
                "SSH private key file is required"
            );
        }

        Ok(())
    }
}
