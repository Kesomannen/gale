use std::{
    io::{Cursor, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    sync::Arc,
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use eyre::{Context, OptionExt, Result, bail, ensure};
use serde::Serialize;
use ssh2::{Error as SshError, ErrorCode, HashType, RenameFlags, Sftp};
use suppaftp::rustls::{
    ClientConfig, DigitallySignedStruct, Error as TlsError, RootCertStore, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use suppaftp::{FtpError, RustlsConnector, RustlsFtpStream, Status};

use super::{
    paths::RemotePath,
    settings::{RemoteAuthentication, RemoteProtocol, RemoteServerSettings},
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const SSH_TIMEOUT: Duration = Duration::from_millis(15_000);
const SFTP_NO_SUCH_FILE: i32 = 2;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ConnectionTestResult {
    Connected {
        fingerprint: Option<String>,
        encrypted: bool,
    },
    HostKeyUntrusted {
        fingerprint: String,
    },
    CertificateUntrusted,
}

pub enum ConnectionAttempt {
    Connected(RemoteConnection),
    HostKeyUntrusted { fingerprint: String },
    CertificateUntrusted,
}

pub struct RemoteConnection {
    client: RemoteClient,
    pub fingerprint: Option<String>,
    pub encrypted: bool,
}

pub struct RemoteEntry {
    pub name: String,
    pub is_directory: bool,
}

enum RemoteClient {
    Sftp { sftp: Sftp, _session: ssh2::Session },
    Ftp(RustlsFtpStream),
}

#[derive(Debug)]
struct TrustAnyCertificate;

impl ServerCertVerifier for TrustAnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        suppaftp::rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub fn test_connection(
    settings: &RemoteServerSettings,
    password: &str,
) -> Result<ConnectionTestResult> {
    match RemoteConnection::connect(settings, password)? {
        ConnectionAttempt::Connected(mut connection) => {
            let directory = settings.server_directory()?;
            connection.check_directory(&directory).with_context(|| {
                format!(
                    "connected successfully, but server directory '{}' could not be accessed",
                    settings.server_directory
                )
            })?;

            Ok(ConnectionTestResult::Connected {
                fingerprint: connection.fingerprint,
                encrypted: connection.encrypted,
            })
        }
        ConnectionAttempt::HostKeyUntrusted { fingerprint } => {
            Ok(ConnectionTestResult::HostKeyUntrusted { fingerprint })
        }
        ConnectionAttempt::CertificateUntrusted => Ok(ConnectionTestResult::CertificateUntrusted),
    }
}

impl RemoteConnection {
    /// Connects and authenticates against the remote server.
    ///
    /// `settings` is expected to have passed [`RemoteServerSettings::validate`]
    /// already; this is the transport layer, so it only parses the paths it
    /// actually needs.
    pub fn connect(settings: &RemoteServerSettings, password: &str) -> Result<ConnectionAttempt> {
        if settings.protocol != RemoteProtocol::Sftp {
            return match Self::connect_ftp(settings, password) {
                Ok(connection) => Ok(ConnectionAttempt::Connected(connection)),
                Err(error)
                    if settings.trusted_invalid_certificate_host.as_deref()
                        != Some(settings.host.trim())
                        && is_untrusted_certificate_error(&error) =>
                {
                    Ok(ConnectionAttempt::CertificateUntrusted)
                }
                Err(error) => Err(error),
            };
        }

        let mut session = connect_tcp(settings)?;
        session.handshake().context("SSH handshake failed")?;

        let fingerprint = host_key_fingerprint(&session)?;

        match settings.trusted_host_key.as_deref() {
            Some(expected) => ensure!(
                expected == fingerprint,
                "SSH host key has changed. Expected {expected}, received {fingerprint}. Refusing to send credentials."
            ),
            None => return Ok(ConnectionAttempt::HostKeyUntrusted { fingerprint }),
        }

        authenticate(&session, settings, password)?;

        let sftp = session
            .sftp()
            .context("connected over SSH, but the server did not provide an SFTP subsystem")?;

        Ok(ConnectionAttempt::Connected(RemoteConnection {
            client: RemoteClient::Sftp {
                sftp,
                _session: session,
            },
            fingerprint: Some(fingerprint),
            encrypted: true,
        }))
    }

    fn connect_ftp(settings: &RemoteServerSettings, password: &str) -> Result<RemoteConnection> {
        ensure!(!password.is_empty(), "FTP password is required");

        let address = format!("{}:{}", settings.host.trim(), settings.port);
        let stream = RustlsFtpStream::connect(address.as_str())
            .with_context(|| format!("failed to connect to {}", settings.host))?;

        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut connector = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        // The user may choose to trust a host whose certificate cannot be
        // verified (self-signed certs are common on game hosts).
        let trust_invalid =
            settings.trusted_invalid_certificate_host.as_deref() == Some(settings.host.trim());
        if trust_invalid {
            connector
                .dangerous()
                .set_certificate_verifier(Arc::new(TrustAnyCertificate));
        }

        let (mut stream, encrypted) = match stream.into_secure(
            RustlsConnector::from(Arc::new(connector)),
            settings.host.trim(),
        ) {
            Ok(stream) => (stream, true),
            // Plain FTP servers reject AUTH TLS entirely; reconnect and stay
            // unencrypted instead of failing.
            Err(error)
                if settings.protocol == RemoteProtocol::Ftp && is_ftp_tls_unsupported(&error) =>
            {
                (
                    RustlsFtpStream::connect(address.as_str())
                        .with_context(|| format!("failed to reconnect to {}", settings.host))?,
                    false,
                )
            }
            Err(error) => return Err(error).context("FTPS TLS negotiation failed"),
        };

        stream
            .login(settings.username.trim(), password)
            .context("FTP authentication failed")?;

        Ok(RemoteConnection {
            client: RemoteClient::Ftp(stream),
            fingerprint: None,
            encrypted,
        })
    }

    pub fn supports_atomic_replace(&self) -> bool {
        matches!(self.client, RemoteClient::Sftp { .. })
    }

    pub fn list_directory_entries(&mut self, path: &RemotePath) -> Result<Vec<RemoteEntry>> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.readdir(Path::new(path.as_str())) {
                Ok(entries) => Ok(entries
                    .into_iter()
                    .filter_map(|(path, stat)| {
                        Some(RemoteEntry {
                            name: path.file_name()?.to_str()?.to_owned(),
                            is_directory: stat.is_dir(),
                        })
                    })
                    .collect()),
                Err(err) if is_sftp_not_found(&err) => Ok(Vec::new()),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => match ftp.list(Some(path.as_str())) {
                Ok(entries) => ftp_list_entries(entries),
                Err(err) if is_ftp_not_found(&err) => Ok(Vec::new()),
                Err(err) => Err(err.into()),
            },
        }
    }

    pub fn check_directory(&mut self, path: &RemotePath) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                sftp.stat(Path::new(path.as_str()))?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                ftp.cwd(path.as_str())?;
                ftp.cwd(original)?;
                Ok(())
            }
        }
    }

    pub fn directory_exists(&mut self, path: &RemotePath) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(Path::new(path.as_str())) {
                Ok(stat) => Ok(stat.is_dir()),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                match ftp.cwd(path.as_str()) {
                    Ok(()) => {
                        ftp.cwd(original)?;
                        Ok(true)
                    }
                    Err(err) if is_ftp_not_found(&err) => {
                        ftp.cwd(original)?;
                        Ok(false)
                    }
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn read_file(&mut self, path: &RemotePath) -> Result<Option<Vec<u8>>> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.open(Path::new(path.as_str())) {
                Ok(mut file) => {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)?;
                    Ok(Some(bytes))
                }
                Err(err) if is_sftp_not_found(&err) => Ok(None),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => match ftp.retr_as_buffer(path.as_str()) {
                Ok(bytes) => Ok(Some(bytes.into_inner())),
                Err(err) if is_ftp_not_found(&err) => Ok(None),
                Err(err) => Err(err.into()),
            },
        }
    }

    pub fn write_file(&mut self, path: &RemotePath, bytes: &[u8]) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                let mut file = sftp.create(Path::new(path.as_str()))?;
                file.write_all(bytes)?;
                file.flush()?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                ftp.put_file(path.as_str(), &mut Cursor::new(bytes))?;
                Ok(())
            }
        }
    }

    pub fn remove_file(&mut self, path: &RemotePath) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.unlink(Path::new(path.as_str())) {
                Ok(()) => Ok(true),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => match ftp.rm(path.as_str()) {
                Ok(()) => Ok(true),
                Err(err) => Err(err.into()),
            },
        }
    }

    pub fn remove_directory(&mut self, path: &RemotePath) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                sftp.rmdir(Path::new(path.as_str()))?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => ftp.rmdir(path.as_str()).map_err(Into::into),
        }
    }

    pub fn ensure_directory(&mut self, path: &RemotePath) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(Path::new(path.as_str())) {
                Ok(_) => Ok(()),
                Err(err) if is_sftp_not_found(&err) => {
                    sftp.mkdir(Path::new(path.as_str()), 0o755)?;
                    Ok(())
                }
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                if ftp.cwd(path.as_str()).is_ok() {
                    ftp.cwd(original)?;
                    return Ok(());
                }
                ftp.cwd(&original)?;
                ftp.mkdir(path.as_str())?;
                Ok(())
            }
        }
    }

    pub fn rename_file(
        &mut self,
        from: &RemotePath,
        to: &RemotePath,
        overwrite: bool,
    ) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                let flags = if overwrite {
                    RenameFlags::ATOMIC | RenameFlags::OVERWRITE | RenameFlags::NATIVE
                } else {
                    RenameFlags::empty()
                };
                sftp.rename(
                    Path::new(from.as_str()),
                    Path::new(to.as_str()),
                    Some(flags),
                )?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                ftp.rename(from.as_str(), to.as_str())?;
                Ok(())
            }
        }
    }

    pub fn file_exists(&mut self, path: &RemotePath) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(Path::new(path.as_str())) {
                Ok(_) => Ok(true),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => match ftp.size(path.as_str()) {
                Ok(_) => Ok(true),
                Err(err) if is_ftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
        }
    }
}

fn connect_tcp(settings: &RemoteServerSettings) -> Result<ssh2::Session> {
    let address = format!("{}:{}", settings.host.trim(), settings.port);
    let addresses = address
        .to_socket_addrs()
        .with_context(|| format!("failed to resolve {}", settings.host))?
        .collect::<Vec<_>>();

    ensure!(!addresses.is_empty(), "host resolved to no addresses");

    let mut last_error = None;

    for address in addresses {
        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(stream) => {
                stream.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
                stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

                let mut session = ssh2::Session::new().context("failed to create SSH session")?;
                session.set_tcp_stream(stream);
                session.set_timeout(SSH_TIMEOUT.as_millis() as u32);

                return Ok(session);
            }
            Err(err) => last_error = Some(err),
        }
    }

    bail!(
        "failed to connect to {}: {}",
        address,
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "unknown network error".to_owned())
    )
}

fn host_key_fingerprint(session: &ssh2::Session) -> Result<String> {
    let hash = session
        .host_key_hash(HashType::Sha256)
        .ok_or_eyre("server did not provide a SHA256 host-key fingerprint")?;

    Ok(format!("SHA256:{}", STANDARD_NO_PAD.encode(hash)))
}

fn authenticate(
    session: &ssh2::Session,
    settings: &RemoteServerSettings,
    credential: &str,
) -> Result<()> {
    match settings.authentication {
        RemoteAuthentication::Password => {
            ensure!(!credential.is_empty(), "SFTP password is required");
            session
                .userauth_password(&settings.username, credential)
                .context("SSH password authentication failed")?;
        }
        RemoteAuthentication::PrivateKey => {
            let private_key = Path::new(&settings.private_key_path);
            ensure!(private_key.is_file(), "SSH private key file does not exist");
            session
                .userauth_pubkey_file(
                    &settings.username,
                    None,
                    private_key,
                    (!credential.is_empty()).then_some(credential),
                )
                .context("SSH private-key authentication failed")?;
        }
        RemoteAuthentication::Agent => session
            .userauth_agent(&settings.username)
            .context("SSH agent authentication failed")?,
    }

    ensure!(session.authenticated(), "SSH authentication was rejected");

    Ok(())
}

fn is_sftp_not_found(error: &SshError) -> bool {
    matches!(error.code(), ErrorCode::SFTP(SFTP_NO_SUCH_FILE))
}

fn is_ftp_not_found(error: &FtpError) -> bool {
    matches!(error, FtpError::UnexpectedResponse(response) if response.status == Status::FileUnavailable)
}

fn is_ftp_tls_unsupported(error: &FtpError) -> bool {
    matches!(error, FtpError::UnexpectedResponse(response) if matches!(response.status, Status::NotImplemented | Status::BadCommand))
}

fn is_untrusted_certificate_error(error: &eyre::Report) -> bool {
    error.chain().any(|cause| {
        if cause
            .downcast_ref::<FtpError>()
            .is_some_and(|error| matches!(error, FtpError::SecureError(_)))
        {
            return true;
        }
        let message = cause.to_string().to_ascii_lowercase();
        message.contains("invalid peer certificate")
            || message.contains("unknownissuer")
            || message.contains("unknown issuer")
    })
}

fn ftp_list_entries(entries: Vec<String>) -> Result<Vec<RemoteEntry>> {
    entries
        .into_iter()
        .map(|entry| {
            entry
                .parse::<suppaftp::list::File>()
                .map(|file| RemoteEntry {
                    name: file.name().to_owned(),
                    is_directory: file.is_directory(),
                })
                .with_context(|| format!("failed to parse FTP LIST entry: {entry}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use suppaftp::{FtpError, Status, types::Response};

    use super::{
        ftp_list_entries, is_ftp_not_found, is_ftp_tls_unsupported, is_untrusted_certificate_error,
    };

    #[test]
    fn parses_ftp_directory_responses() {
        let entries = ftp_list_entries(vec![
            "-rw-r--r-- 1 owner group 42 Sep 11 12:00 File name.dll.old".to_owned(),
        ])
        .unwrap();
        assert_eq!(entries[0].name, "File name.dll.old");
        assert!(!entries[0].is_directory);
    }

    #[test]
    fn recognizes_missing_ftp_paths() {
        let missing = FtpError::UnexpectedResponse(Response::new(
            Status::from(550),
            b"550 path does not exists".to_vec(),
        ));
        assert!(is_ftp_not_found(&missing));
    }

    #[test]
    fn recognizes_ftp_servers_without_tls() {
        let unsupported = FtpError::UnexpectedResponse(Response::new(
            Status::from(502),
            b"502 AUTH TLS not implemented".to_vec(),
        ));

        assert!(is_ftp_tls_unsupported(&unsupported));
    }

    #[test]
    fn recognizes_rustls_unknown_issuers() {
        let error = eyre::eyre!("Connection error: invalid peer certificate: UnknownIssuer");

        assert!(is_untrusted_certificate_error(&error));
    }
}
