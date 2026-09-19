use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

use eyre::{Context, Result, eyre};
use futures_util::StreamExt;
use http::header;
use reqwest::{StatusCode, header::HeaderValue};
use tauri::AppHandle;
use tracing::{debug, warn};

use crate::state::ManagerExt;

use super::{
    InstallError, InstallEvent, InstallOptions, InstallResult, InstallTask, ModInstall,
    check_cancel, emit,
};

/// Downloads the archive of the given mod into memory, retrying (and resuming) if
/// the connection is interrupted.
pub(super) async fn download(
    install: &ModInstall,
    cancel: &AtomicBool,
    options: &InstallOptions,
    app: &AppHandle,
) -> InstallResult<Vec<u8>> {
    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Download),
        app,
    );

    let url = install.id.backend.download_url(&install.ident);

    debug!(
        ident = %install.ident,
        size = install.file_size,
        url = %url,
        "downloading mod"
    );

    // how many times in a row a download may fail without making any progress
    // before we give up
    const MAX_RETRIES: usize = 5;
    // hard limit on the total number of attempts, so that we don't retry forever
    // if the connection keeps getting dropped right after receiving some data
    const MAX_ATTEMPTS: usize = 20;
    const INITIAL_BACKOFF: Duration = Duration::from_secs(2);
    const MAX_BACKOFF: Duration = Duration::from_secs(30);

    let mut response = Vec::with_capacity(install.file_size as usize);
    let mut retries = 0;
    let mut attempts = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        let downloaded = response.len();
        attempts += 1;

        match try_download(&mut response, &url, cancel, options, app).await {
            Ok(()) => break Ok(response),
            Err(InstallError::Cancelled) => return Err(InstallError::Cancelled),
            Err(InstallError::Error(err)) => {
                // errors like a 404 won't resolve themselves, so retrying is pointless
                if is_fatal_download_error(&err) {
                    break Err(InstallError::Error(err));
                }

                // as long as the download keeps making progress, the connection itself
                // is working, so keep retrying from where it left off
                if response.len() > downloaded {
                    debug!(
                        downloaded = response.len(),
                        total = install.file_size,
                        "download interrupted, will resume where it left off"
                    );

                    retries = 0;
                    backoff = INITIAL_BACKOFF;
                } else {
                    retries += 1;
                }

                if retries >= MAX_RETRIES || attempts >= MAX_ATTEMPTS {
                    break Err(InstallError::Error(err.wrap_err("max retries exceeded")));
                }

                warn!(
                    attempt = attempts,
                    downloaded = response.len(),
                    total = install.file_size,
                    err = ?err,
                    url = %url,
                    backoff = ?backoff,
                    "download failed, retrying"
                );

                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }
}

/// Downloads the file at `url` into `buf`, resuming from the end of `buf` if it
/// already contains (partial) data.
///
/// Errors out if the server starts sending at an unexpected offset, or if the
/// connection is closed before the whole file was received, so that the caller can
/// retry the download where it left off.
async fn try_download(
    buf: &mut Vec<u8>,
    url: &str,
    cancel: &AtomicBool,
    options: &InstallOptions,
    app: &AppHandle,
) -> InstallResult<()> {
    const UPDATE_DELAY: Duration = Duration::from_millis(100);

    let (response, expected_len) = match send_download_request(buf, url, app).await? {
        Some((res, len)) => (res, len),
        None => return Ok(()), // download already complete
    };

    let mut stream = response.bytes_stream();
    let mut last_update = Instant::now();
    let mut last_size_update = buf.len() as i64;

    while let Some(item) = stream.next().await {
        let item = item.context("failed to read chunk from stream")?;
        buf.extend_from_slice(&item);

        if last_update.elapsed() >= UPDATE_DELAY {
            last_update = Instant::now();
            emit(
                InstallEvent::AddProgress {
                    mods: 0,
                    bytes: buf.len() as i64 - last_size_update,
                },
                app,
            );
            last_size_update = buf.len() as i64;

            check_cancel(cancel, options)?;
        }
    }

    emit(
        InstallEvent::AddProgress {
            mods: 0,
            bytes: buf.len() as i64 - last_size_update,
        },
        app,
    );

    // the connection may have been closed cleanly before all data was sent
    if let Some(expected_len) = expected_len
        && buf.len() as u64 != expected_len
    {
        return Err(eyre!(
            "incomplete download: received {} out of {} bytes",
            buf.len(),
            expected_len
        )
        .into());
    }

    Ok(())
}

async fn send_download_request(
    buf: &mut Vec<u8>,
    url: &str,
    app: &AppHandle,
) -> Result<Option<(reqwest::Response, Option<u64>)>, InstallError> {
    let resumed_from = buf.len();

    let mut request = app
        .http()
        .get(url)
        // ask for an uncompressed body, since we can't resume an archive from the
        // byte offsets of a compressed response
        .header(header::ACCEPT_ENCODING, "identity");

    if resumed_from > 0 {
        request = request.header(header::RANGE, format!("bytes={resumed_from}-"));
    }

    let response = request.send().await.context("failed to send request")?;
    let status = response.status();
    let version = response.version();

    debug!(url = %url, resumed_from, %status, ?version, "download requested");

    let response = match status {
        StatusCode::PARTIAL_CONTENT if resumed_from > 0 => response,
        // the server has no more data to give us, so the download is already complete
        StatusCode::RANGE_NOT_SATISFIABLE if resumed_from > 0 => {
            let total = response
                .headers()
                .get(header::CONTENT_RANGE)
                .map(parse_content_range)
                .and_then(|range| range.total);

            if total == Some(resumed_from as u64) {
                debug!(url = %url, downloaded = resumed_from, "download already complete");
                return Ok(None);
            }

            discard_partial_download(buf, app);
            return Err(eyre!("server rejected range request at offset {resumed_from}").into());
        }
        // the server ignored the range request, so the whole file must be downloaded again
        StatusCode::OK if resumed_from > 0 => {
            warn!(
                url = %url,
                downloaded = resumed_from,
                "server ignored range request, restarting download"
            );

            discard_partial_download(buf, app);
            response
        }
        _ => response.error_for_status().context("request failed")?,
    };

    let expected_start = buf.len() as u64;

    let content_range = response
        .headers()
        .get(header::CONTENT_RANGE)
        .map(parse_content_range);

    if let Some(start) = content_range.and_then(|range| range.start)
        && start != expected_start
    {
        discard_partial_download(buf, app);
        return Err(eyre!("unexpected download offset {start}, expected {expected_start}").into());
    }

    let expected_len = content_range
        .and_then(|range| range.total)
        .or_else(|| {
            // if the body isn't compressed, the content length must match the data we receive
            if response.headers().contains_key(header::CONTENT_ENCODING) {
                // body is compressed and the content range wasn't provided; we can't know the total size of the request
                None
            } else {
                response.content_length()
            }
        })
        .map(|len| len + expected_start);

    Ok(Some((response, expected_len)))
}

/// The parts of a [`CONTENT_RANGE`] header that we care about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContentRange {
    start: Option<u64>,
    total: Option<u64>,
}

/// Parses a `Content-Range` header like `bytes 0-1023/2048` or `bytes */2048`.
fn parse_content_range(value: &HeaderValue) -> ContentRange {
    let value = value.to_str().unwrap_or_default();
    let value = value.strip_prefix("bytes ").unwrap_or(value);
    let (bounds, total) = value.split_once('/').unwrap_or((value, ""));

    let start = bounds.split_once('-').map_or(bounds, |(start, _)| start);

    ContentRange {
        start: start.trim().parse().ok(),
        total: total.trim().parse().ok(),
    }
}

/// Clears a partially downloaded file, undoing its contribution to the install progress.
fn discard_partial_download(buf: &mut Vec<u8>, app: &AppHandle) {
    if buf.is_empty() {
        return;
    }

    emit(
        InstallEvent::AddProgress {
            mods: 0,
            bytes: -(buf.len() as i64),
        },
        app,
    );

    buf.clear();
}

/// Whether an error from [`try_download`] is permanent, in which case retrying is pointless.
fn is_fatal_download_error(err: &eyre::Report) -> bool {
    let Some(status) = err
        .downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status)
    else {
        return false;
    };

    // timeouts and rate limits are worth retrying, unlike (for example) 404s
    status.is_client_error()
        && status != StatusCode::REQUEST_TIMEOUT
        && status != StatusCode::TOO_MANY_REQUESTS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> ContentRange {
        parse_content_range(&HeaderValue::from_str(value).unwrap())
    }

    #[test]
    fn start_and_total() {
        assert_eq!(
            parse("bytes 0-1023/2048"),
            ContentRange {
                start: Some(0),
                total: Some(2048)
            }
        );
    }

    #[test]
    fn resumed_download() {
        // what the thunderstore CDN responds with when resuming a download
        assert_eq!(
            parse("bytes 1024-1055/778380268"),
            ContentRange {
                start: Some(1024),
                total: Some(778380268)
            }
        );
    }

    #[test]
    fn unsatisfiable_range() {
        // a 416 response only tells us the total size of the file
        assert_eq!(
            parse("bytes */2048"),
            ContentRange {
                start: None,
                total: Some(2048)
            }
        );
    }

    #[test]
    fn unknown_total() {
        assert_eq!(
            parse("bytes 1000-1999/*"),
            ContentRange {
                start: Some(1000),
                total: None
            }
        );
    }

    #[test]
    fn lenient_formatting() {
        let expected = ContentRange {
            start: Some(0),
            total: Some(2048),
        };

        // missing "bytes " prefix
        assert_eq!(parse("0-1023/2048"), expected);

        // stray whitespace around the offsets
        assert_eq!(parse("bytes 0-1023 / 2048"), expected);
    }

    #[test]
    fn invalid() {
        let empty = ContentRange {
            start: None,
            total: None,
        };

        assert_eq!(parse(""), empty);
        assert_eq!(parse("bytes"), empty);
        assert_eq!(parse("bytes abc-def/ghi"), empty);
    }
}
