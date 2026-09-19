use std::{
    borrow::Cow,
    io::{Cursor, Read},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
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
    crypto::{verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use suppaftp::{FtpError, RustlsConnector, RustlsFtpStream, Status};

use super::config::{RemoteAuthentication, RemoteProtocol, RemoteServerSettings};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const SSH_TIMEOUT: Duration = Duration::from_secs(15);
const SFTP_NO_SUCH_FILE: i32 = 2;

pub async fn run_blocking<T, F>(operation: &'static str, task: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|err| eyre::eyre!("{operation} worker failed: {err}"))?
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ConnectionTestResult {
    Connected { encrypted: bool },
    HostKeyUntrusted { fingerprint: String },
    CertificateUntrusted,
}

pub enum ConnectionAttempt {
    Connected(RemoteConnection),
    HostKeyUntrusted { fingerprint: String },
    CertificateUntrusted,
}

pub struct RemoteConnection {
    client: RemoteClient,
    pub encrypted: bool,
}

pub struct RemoteEntry {
    pub name: PathBuf,
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
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &suppaftp::rustls::crypto::aws_lc_rs::default_provider()
                .signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, TlsError> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &suppaftp::rustls::crypto::aws_lc_rs::default_provider()
                .signature_verification_algorithms,
        )
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
    match connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => {
            let mut connection = connection;
            connection
                .check_directory(&settings.server_directory)
                .with_context(|| {
                    format!(
                        "connected successfully, but server directory '{}' could not be accessed",
                        settings.server_directory.display()
                    )
                })?;

            Ok(ConnectionTestResult::Connected {
                encrypted: connection.encrypted,
            })
        }
        ConnectionAttempt::HostKeyUntrusted { fingerprint } => {
            Ok(ConnectionTestResult::HostKeyUntrusted { fingerprint })
        }
        ConnectionAttempt::CertificateUntrusted => Ok(ConnectionTestResult::CertificateUntrusted),
    }
}

pub fn connect(settings: &RemoteServerSettings, password: &str) -> Result<ConnectionAttempt> {
    if settings.protocol != RemoteProtocol::Sftp {
        return match RemoteConnection::connect_ftp(settings, password) {
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
        encrypted: true,
    }))
}

impl RemoteConnection {
    fn connect_ftp(settings: &RemoteServerSettings, password: &str) -> Result<Self> {
        settings.validate_credential(password)?;

        let address = format!("{}:{}", settings.host.trim(), settings.port);
        let stream = RustlsFtpStream::connect(address.as_str())
            .with_context(|| format!("failed to connect to {}", settings.host))?;
        let trust_invalid =
            settings.trusted_invalid_certificate_host.as_deref() == Some(settings.host.trim());
        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut connector = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

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

        Ok(Self {
            client: RemoteClient::Ftp(stream),
            encrypted,
        })
    }

    pub fn supports_atomic_replace(&self) -> bool {
        matches!(self.client, RemoteClient::Sftp { .. })
    }

    pub fn list_directory_entries(&mut self, path: &Path) -> Result<Vec<RemoteEntry>> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.readdir(path) {
                Ok(entries) => Ok(entries
                    .into_iter()
                    .filter_map(|(path, stat)| {
                        Some(RemoteEntry {
                            name: path.file_name()?.into(),
                            is_directory: stat.is_dir(),
                        })
                    })
                    .collect()),
                Err(err) if is_sftp_not_found(&err) => Ok(Vec::new()),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let path = remote_path(path)?;

                match ftp.list(Some(path.as_ref())) {
                    Ok(entries) => ftp_list_entries(entries),
                    Err(err) if is_ftp_not_found(&err) => Ok(Vec::new()),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn check_directory(&mut self, path: &Path) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                sftp.stat(path)?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                ftp.cwd(remote_path(path)?.as_ref())?;
                ftp.cwd(original)?;
                Ok(())
            }
        }
    }

    pub fn directory_exists(&mut self, path: &Path) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(path) {
                Ok(stat) => Ok(stat.is_dir()),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                match ftp.cwd(remote_path(path)?.as_ref()) {
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

    pub fn read_file(&mut self, path: &Path) -> Result<Option<Vec<u8>>> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.open(path) {
                Ok(mut file) => {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)?;
                    Ok(Some(bytes))
                }
                Err(err) if is_sftp_not_found(&err) => Ok(None),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let path = remote_path(path)?;

                match ftp.retr_as_buffer(path.as_ref()) {
                    Ok(bytes) => Ok(Some(bytes.into_inner())),
                    Err(err) if is_ftp_not_found(&err) => Ok(None),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn write_file(&mut self, path: &Path, bytes: &[u8]) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                use std::io::Write;
                let mut file = sftp.create(path)?;
                file.write_all(bytes)?;
                file.flush()?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                ftp.put_file(remote_path(path)?.as_ref(), &mut Cursor::new(bytes))?;
                Ok(())
            }
        }
    }

    pub fn remove_file(&mut self, path: &Path) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.unlink(path) {
                Ok(()) => Ok(true),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let path = remote_path(path)?;

                match ftp.rm(path.as_ref()) {
                    Ok(()) => Ok(true),
                    Err(err) if is_ftp_not_found(&err) => Ok(false),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn remove_directory(&mut self, path: &Path) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.rmdir(path) {
                Ok(()) => Ok(()),
                Err(err) if is_sftp_not_found(&err) => Ok(()),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let path = remote_path(path)?;
                match ftp.rmdir(path.as_ref()) {
                    Ok(()) => Ok(()),
                    Err(err) if is_ftp_not_found(&err) => Ok(()),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn ensure_directory(&mut self, path: &Path) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(path) {
                Ok(_) => Ok(()),
                Err(err) if is_sftp_not_found(&err) => sftp.mkdir(path, 0o755).map_err(Into::into),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let original = ftp.pwd()?;
                let path = remote_path(path)?;
                if ftp.cwd(path.as_ref()).is_ok() {
                    ftp.cwd(original)?;
                    return Ok(());
                }
                ftp.cwd(&original)?;
                ftp.mkdir(path.as_ref())?;
                Ok(())
            }
        }
    }

    pub fn rename_file(&mut self, from: &Path, to: &Path, overwrite: bool) -> Result<()> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => {
                let flags = if overwrite {
                    RenameFlags::ATOMIC | RenameFlags::OVERWRITE | RenameFlags::NATIVE
                } else {
                    RenameFlags::empty()
                };
                sftp.rename(from, to, Some(flags))?;
                Ok(())
            }
            RemoteClient::Ftp(ftp) => {
                let from = remote_path(from)?;
                let to = remote_path(to)?;
                ftp.rename(from.as_ref(), to.as_ref())?;
                Ok(())
            }
        }
    }

    pub fn file_exists(&mut self, path: &Path) -> Result<bool> {
        match &mut self.client {
            RemoteClient::Sftp { sftp, .. } => match sftp.stat(path) {
                Ok(_) => Ok(true),
                Err(err) if is_sftp_not_found(&err) => Ok(false),
                Err(err) => Err(err.into()),
            },
            RemoteClient::Ftp(ftp) => {
                let path = remote_path(path)?;

                match ftp.size(path.as_ref()) {
                    Ok(_) => Ok(true),
                    Err(err) if is_ftp_not_found(&err) => Ok(false),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }
}

fn remote_path(path: &Path) -> Result<Cow<'_, str>> {
    let path = path
        .to_str()
        .ok_or_else(|| eyre::eyre!("remote path is not valid Unicode"))?;

    if path.contains('\\') {
        Ok(Cow::Owned(path.replace('\\', "/")))
    } else {
        Ok(Cow::Borrowed(path))
    }
}

fn is_sftp_not_found(error: &SshError) -> bool {
    matches!(error.code(), ErrorCode::SFTP(SFTP_NO_SUCH_FILE))
}

fn is_ftp_not_found(error: &FtpError) -> bool {
    let FtpError::UnexpectedResponse(response) = error else {
        return false;
    };
    if response.status != Status::FileUnavailable {
        return false;
    }

    let message = String::from_utf8_lossy(&response.body).to_ascii_lowercase();
    !message.contains("permission denied")
        && !message.contains("access denied")
        && !message.contains("not permitted")
}

fn is_ftp_tls_unsupported(error: &FtpError) -> bool {
    matches!(error, FtpError::UnexpectedResponse(response) if matches!(response.status, Status::NotImplemented | Status::BadCommand))
}

fn is_untrusted_certificate_error(error: &eyre::Report) -> bool {
    error.chain().any(|cause| {
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
                    name: file.name().into(),
                    is_directory: file.is_directory(),
                })
                .with_context(|| format!("failed to parse FTP LIST entry: {entry}"))
        })
        .collect()
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
                session.set_timeout(SSH_TIMEOUT.as_millis().try_into().unwrap());

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
    settings.validate_credential(credential)?;

    match settings.authentication {
        RemoteAuthentication::Password => {
            session
                .userauth_password(&settings.username, credential)
                .context("SSH password authentication failed")?;
        }
        RemoteAuthentication::PrivateKey => {
            ensure!(
                settings.private_key_path.is_file(),
                "SSH private key file does not exist"
            );
            session
                .userauth_pubkey_file(
                    &settings.username,
                    None,
                    &settings.private_key_path,
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use suppaftp::{FtpError, Status, types::Response};

    use super::{
        ftp_list_entries, is_ftp_not_found, is_ftp_tls_unsupported, is_untrusted_certificate_error,
        remote_path,
    };

    #[test]
    fn converts_paths_at_the_transport_boundary() {
        assert_eq!(
            remote_path(Path::new(r"\plugins\Test.dll")).unwrap(),
            "/plugins/Test.dll"
        );
    }

    #[test]
    fn parses_ftp_directory_responses() {
        let entries = ftp_list_entries(vec![
            "-rw-r--r-- 1 owner group 42 Sep 11 12:00 File name.dll.old".to_owned(),
        ])
        .unwrap();
        assert_eq!(entries[0].name, PathBuf::from("File name.dll.old"));
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
    fn preserves_ftp_permission_errors() {
        let denied = FtpError::UnexpectedResponse(Response::new(
            Status::from(550),
            b"550 Permission denied".to_vec(),
        ));

        assert!(!is_ftp_not_found(&denied));
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

    #[test]
    fn preserves_non_certificate_tls_errors() {
        let error = eyre::eyre!("Secure error: peer closed the connection");

        assert!(!is_untrusted_certificate_error(&error));
    }
}
