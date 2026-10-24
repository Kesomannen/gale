use std::path::PathBuf;

use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize};
use tauri::AppHandle;

use crate::state::ManagerExt;

const MIN_PASSWORD_LENGTH: usize = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerLocation {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteAuthentication {
    Password,
    PrivateKey,
    Agent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteProtocol {
    Sftp,
    #[serde(alias = "ftps")]
    Ftp,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileServerSettings {
    pub location: ServerLocation,
    pub local: LocalServerSettings,
    pub remote: RemoteServerSettings,
}

pub fn get(app: &AppHandle) -> Option<ProfileServerSettings> {
    app.lock_manager().active_profile().server_settings.clone()
}

pub fn save_for_profile(
    app: &AppHandle,
    profile_id: i64,
    settings: ProfileServerSettings,
) -> Result<()> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;
    profile.server_settings = Some(settings);
    profile.save(app, false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LocalServerSettings {
    Valheim {
        server_name: String,
        world: String,
        port: u16,
        public_server: bool,
        crossplay: bool,
        extra_args: String,
    },
}

impl LocalServerSettings {
    pub fn validate(&self, password: &str) -> Result<()> {
        match self {
            Self::Valheim {
                server_name,
                world,
                port,
                public_server,
                ..
            } => {
                ensure!(
                    !server_name.trim().is_empty(),
                    "server name cannot be empty"
                );
                ensure!(!world.trim().is_empty(), "world name cannot be empty");
                ensure!(*port != 0, "server port cannot be 0");

                if *public_server {
                    ensure!(
                        password.encode_utf16().count() >= MIN_PASSWORD_LENGTH,
                        "server password must contain at least {MIN_PASSWORD_LENGTH} characters"
                    );

                    ensure!(
                        !world.contains(password),
                        "server password cannot be contained in the world name"
                    );
                }
            }
        }

        Ok(())
    }

    pub fn extra_args(&self) -> &str {
        match self {
            Self::Valheim { extra_args, .. } => extra_args,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteServerSettings {
    pub protocol: RemoteProtocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(deserialize_with = "deserialize_server_directory")]
    pub server_directory: PathBuf,
    pub authentication: RemoteAuthentication,
    pub private_key_path: PathBuf,
    pub trusted_host_key: Option<String>,
    pub trusted_invalid_certificate_host: Option<String>,
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
            !self.server_directory.as_os_str().is_empty(),
            "remote server directory cannot be empty"
        );
        if self.protocol == RemoteProtocol::Sftp
            && self.authentication == RemoteAuthentication::PrivateKey
        {
            ensure!(
                !self.private_key_path.as_os_str().is_empty(),
                "SSH private key file is required"
            );
        }

        Ok(())
    }

    pub fn validate_credential(&self, credential: &str) -> Result<()> {
        if self.protocol != RemoteProtocol::Sftp {
            ensure!(!credential.is_empty(), "FTP password is required");
        } else if self.authentication == RemoteAuthentication::Password {
            ensure!(!credential.is_empty(), "SFTP password is required");
        }

        Ok(())
    }
}

fn deserialize_server_directory<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = PathBuf::deserialize(deserializer)?;

    Ok(if path.as_os_str().is_empty() {
        PathBuf::from("/")
    } else {
        path
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentProfileServerSettings {
    location: ServerLocation,
    local: LocalServerSettings,
    remote: RemoteServerSettings,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProfileServerSettings {
    location: ServerLocation,
    server_name: String,
    world: String,
    port: u16,
    public_server: bool,
    crossplay: bool,
    extra_args: String,
    remote: RemoteServerSettings,
}

impl<'de> Deserialize<'de> for ProfileServerSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StoredSettings {
            Current(CurrentProfileServerSettings),
            Legacy(LegacyProfileServerSettings),
        }

        Ok(match StoredSettings::deserialize(deserializer)? {
            StoredSettings::Current(settings) => Self {
                location: settings.location,
                local: settings.local,
                remote: settings.remote,
            },
            StoredSettings::Legacy(settings) => Self {
                location: settings.location,
                local: LocalServerSettings::Valheim {
                    server_name: settings.server_name,
                    world: settings.world,
                    port: settings.port,
                    public_server: settings.public_server,
                    crossplay: settings.crossplay,
                    extra_args: settings.extra_args,
                },
                remote: settings.remote,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::{LocalServerSettings, ProfileServerSettings, RemoteProtocol, ServerLocation};

    #[test]
    fn round_trips_current_settings() {
        let value = json!({
            "location": "local",
            "local": {
                "type": "valheim",
                "serverName": "Test Server",
                "world": "Test World",
                "port": 2456,
                "publicServer": true,
                "crossplay": false,
                "extraArgs": ""
            },
            "remote": {
                "protocol": "sftp",
                "host": "example.com",
                "port": 22,
                "username": "user",
                "serverDirectory": "/",
                "authentication": "password",
                "privateKeyPath": "",
                "trustedHostKey": null,
                "trustedInvalidCertificateHost": null
            }
        });

        let settings: ProfileServerSettings = serde_json::from_value(value.clone()).unwrap();

        assert_eq!(serde_json::to_value(settings).unwrap(), value);
    }

    #[test]
    fn loads_legacy_settings() {
        let settings: ProfileServerSettings = serde_json::from_value(json!({
            "location": "remote",
            "serverName": "Test Server",
            "world": "Test World",
            "port": 2456,
            "publicServer": true,
            "crossplay": false,
            "extraArgs": "",
            "remote": {
                "protocol": "ftps",
                "host": "example.com",
                "port": 22,
                "username": "user",
                "serverDirectory": "",
                "authentication": "password",
                "privateKeyPath": "",
                "trustedHostKey": null,
                "trustedInvalidCertificateHost": null
            }
        }))
        .unwrap();

        assert_eq!(settings.location, ServerLocation::Remote);
        assert!(matches!(
            settings.local,
            LocalServerSettings::Valheim {
                server_name,
                world,
                ..
            } if server_name == "Test Server" && world == "Test World"
        ));
        assert_eq!(settings.remote.protocol, RemoteProtocol::Ftp);
        assert_eq!(settings.remote.server_directory, PathBuf::from("/"));
    }

    #[test]
    fn validates_valheim_public_server_password() {
        let settings = LocalServerSettings::Valheim {
            server_name: "Test Server".to_owned(),
            world: "GaleTestingWorld".to_owned(),
            port: 2456,
            public_server: true,
            crossplay: false,
            extra_args: String::new(),
        };

        assert!(settings.validate("").is_err());
        assert!(settings.validate("Testing").is_err());
        assert!(settings.validate("valid password").is_ok());
    }

    #[test]
    fn allows_valheim_private_server_without_password() {
        let settings = LocalServerSettings::Valheim {
            server_name: "Test Server".to_owned(),
            world: "Test World".to_owned(),
            port: 2456,
            public_server: false,
            crossplay: false,
            extra_args: String::new(),
        };

        assert!(settings.validate("").is_ok());
    }
}
