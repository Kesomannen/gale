//! BepInEx launch support for macOS.
//!
//! Doorstop arguments alone do nothing on macOS: the game must be started with
//! `DYLD_INSERT_LIBRARIES` pointing at a doorstop dylib. Gale keeps two files in
//! the profile for that: its own launcher script (`run_bepinex.sh`) which sets
//! up the environment and execs the game, and a universal (x86_64 + arm64)
//! `libdoorstop.dylib` from the UnityDoorstop release, since the one shipped by
//! BepInExPack is x86_64-only and can never inject into an Apple Silicon game.
//!
//! Nothing is copied into the game directory; Steam runs the launcher through
//! the game's launch options (`/bin/sh "<profile>/run_bepinex.sh" %command%`) and Gale's
//! direct launch mode runs it itself.

use std::{
    fs,
    io::{Cursor, Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, Result, bail, ensure, eyre};
use reqwest_middleware::ClientWithMiddleware;
use tauri::AppHandle;
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::{prefs::Prefs, profile::ManagedGame, state::ManagerExt, util::error::IoResultExt};

pub const DOORSTOP_VERSION: &str = "4.5.0";
const DOORSTOP_URL: &str =
    "https://github.com/NeighTools/UnityDoorstop/releases/download/v4.5.0/doorstop_macos_release_4.5.0.zip";
const DOORSTOP_ZIP_ENTRY: &str = "universal/libdoorstop.dylib";
/// Upper bound on the dylib we are willing to extract; the real one is ~100 KiB.
const MAX_DYLIB_SIZE: u64 = 8 * 1024 * 1024;
/// blake3 of `universal/libdoorstop.dylib` inside the release zip above.
const DOORSTOP_BLAKE3: &str = "53f5d52514f06b521239aff51f633fe6ff9ee9f673be99b08b84e9cb7f145eec";

const DYLIB_NAME: &str = "libdoorstop.dylib";
const VERSION_FILE: &str = ".doorstop_version";
const LAUNCHER_NAME: &str = "run_bepinex.sh";
const LAUNCHER_SCRIPT: &str = include_str!("../../../assets/macos/run_bepinex.sh");

pub fn launcher_path(profile_dir: &Path) -> PathBuf {
    profile_dir.join(LAUNCHER_NAME)
}

fn dylib_path(profile_dir: &Path) -> PathBuf {
    profile_dir.join(DYLIB_NAME)
}

fn version_path(profile_dir: &Path) -> PathBuf {
    profile_dir.join(VERSION_FILE)
}

/// Writes Gale's launcher script to `<profile>/run_bepinex.sh`, replacing
/// whatever is there (the file is Gale-owned), and makes it executable.
pub fn ensure_launcher(profile_dir: &Path) -> Result<()> {
    let path = launcher_path(profile_dir);

    write_replacing(&path, LAUNCHER_SCRIPT)?;
    fs::set_permissions(&path, PermissionsExt::from_mode(0o755))
        .fs_context("making launcher executable", &path)?;

    debug!(path = %path.display(), "wrote macOS launcher");

    Ok(())
}

/// Makes sure `<profile>/libdoorstop.dylib` is the pinned universal doorstop
/// build and `<profile>/.doorstop_version` says so, downloading the release
/// with the app's HTTP client if needed.
pub async fn ensure_doorstop(profile_dir: &Path, app: &AppHandle) -> Result<()> {
    ensure_doorstop_with(profile_dir, app.http()).await
}

/// [`ensure_doorstop`] for callers without an [`AppHandle`].
pub async fn ensure_doorstop_with(profile_dir: &Path, http: &ClientWithMiddleware) -> Result<()> {
    let dylib_path = dylib_path(profile_dir);
    let version_path = version_path(profile_dir);

    if doorstop_installed(profile_dir) {
        debug!("doorstop {} is already present", DOORSTOP_VERSION);
        return Ok(());
    }

    if dylib_path.exists() {
        // the dylib is ours, only the version file was replaced
        // (BepInExPack writes its own when it is installed or updated)
        let dylib = fs::read(&dylib_path).fs_context("reading doorstop", &dylib_path)?;
        if verify_hash(&dylib).is_ok() {
            info!("restoring doorstop version file to {}", DOORSTOP_VERSION);
            write_replacing(&version_path, DOORSTOP_VERSION)?;
            return Ok(());
        }
    }

    info!(
        version = DOORSTOP_VERSION,
        url = DOORSTOP_URL,
        "downloading universal doorstop"
    );

    let bytes = http
        .get(DOORSTOP_URL)
        .send()
        .await
        .context("failed to request doorstop release")?
        .error_for_status()
        .context("doorstop release request failed")?
        .bytes()
        .await
        .context("failed to download doorstop release")?;

    let dylib = extract_dylib(&bytes).context("failed to extract doorstop release")?;
    verify_hash(&dylib)?;

    write_replacing(&dylib_path, &dylib)?;
    write_replacing(&version_path, DOORSTOP_VERSION)?;

    info!(
        path = %dylib_path.display(),
        "installed doorstop {} ({} bytes)",
        DOORSTOP_VERSION,
        dylib.len()
    );

    Ok(())
}

/// Whether `<profile>/libdoorstop.dylib` is present and the version file says it
/// is the pinned build; the fast path that needs no download.
pub fn doorstop_installed(profile_dir: &Path) -> bool {
    dylib_path(profile_dir).exists()
        && fs::read_to_string(version_path(profile_dir))
            .is_ok_and(|version| version.trim() == DOORSTOP_VERSION)
}

/// Runs [`ensure_doorstop`] to completion from synchronous code. The future is
/// driven on a separate thread, so this is safe to call from a tokio worker.
fn ensure_doorstop_blocking(profile_dir: &Path, app: &AppHandle) -> Result<()> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| tauri::async_runtime::block_on(ensure_doorstop(profile_dir, app)))
            .join()
            .map_err(|_| eyre!("doorstop download thread panicked"))?
    })
}

/// Synchronous safety net for callers that launch while holding the prefs and
/// manager locks (the CLI, including a second instance started with
/// `--launch`): checks the game's signature, re-signing it if the user has
/// allowed that, then fills in the launcher and doorstop if they are missing.
/// The launch command runs [`prepare_launch`] before it takes any lock, so on
/// launches from the UI this only re-reads the signature (milliseconds) and
/// finds nothing else to do. The caller passes the prefs it already holds;
/// locking them again here would deadlock.
pub fn ensure_ready_blocking(
    game: &ManagedGame,
    game_dir: &Path,
    prefs: &Prefs,
    app: &AppHandle,
) -> Result<()> {
    ensure_injectable(game.game.name, game_dir, prefs.macos_allow_resign)?;

    let profile_dir = &game.active_profile().path;

    if !launcher_path(profile_dir).is_file() {
        ensure_launcher(profile_dir).context("failed to write BepInEx launcher")?;
    }

    if !doorstop_installed(profile_dir) {
        ensure_doorstop_blocking(profile_dir, app)
            .context("failed to set up doorstop, which BepInEx needs to load on macOS")?;
    }

    Ok(())
}

/// Everything a modded BepInEx launch needs on macOS that may take a while:
/// the launcher script and doorstop in the profile, and a game signature that
/// lets doorstop inject. Takes the prefs and manager locks only long enough to
/// copy out what it needs, so a first-time doorstop download or a re-sign never
/// stalls the rest of the app. Does nothing for vanilla or non-BepInEx launches.
pub async fn prepare_launch(vanilla: bool, app: &AppHandle) -> Result<()> {
    if vanilla {
        return Ok(());
    }

    let (game_name, game_dir, profile_dir, allow_resign) = {
        let prefs = app.lock_prefs();
        let manager = app.lock_manager();
        let game = manager.active_game();

        if !game.uses_bepinex() {
            return Ok(());
        }

        let game_dir =
            super::locate_game_dir(game.game, &prefs).context("failed to locate game directory")?;

        (
            game.game.name.to_owned(),
            game_dir,
            game.active_profile().path.clone(),
            prefs.macos_allow_resign,
        )
    };

    tauri::async_runtime::spawn_blocking(move || {
        ensure_injectable(&game_name, &game_dir, allow_resign)
    })
    .await
    .map_err(|err| eyre!("signature check did not complete: {err}"))??;

    ensure_launcher(&profile_dir).context("failed to write BepInEx launcher")?;
    ensure_doorstop(&profile_dir, app)
        .await
        .context("failed to set up doorstop, which BepInEx needs to load on macOS")?;

    Ok(())
}

// --- code signing -----------------------------------------------------------
//
// Games shipped with Apple's hardened runtime are killed by the kernel as soon
// as Mono starts writing JIT code, unless the signature also carries the
// `allow-unsigned-executable-memory` entitlement. macOS then reports the game
// as "damaged". Even with that, the hardened runtime ignores `DYLD_*` without
// `allow-dyld-environment-variables` and refuses a dylib that is not signed by
// the game's team without `disable-library-validation`, so doorstop silently
// never loads. The community fix (what r2modmac does silently) is to replace
// the signature with an ad-hoc one, which has no hardened runtime at all.

const ENTITLEMENT_UNSIGNED_EXEC_MEMORY: &str =
    "com.apple.security.cs.allow-unsigned-executable-memory";
const ENTITLEMENT_DYLD_ENV: &str = "com.apple.security.cs.allow-dyld-environment-variables";
const ENTITLEMENT_DISABLE_LIBRARY_VALIDATION: &str =
    "com.apple.security.cs.disable-library-validation";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SigningStatus {
    /// The `runtime` flag is set on the code directory.
    pub hardened_runtime: bool,
    /// `com.apple.security.cs.allow-unsigned-executable-memory` is granted.
    pub allows_unsigned_exec_memory: bool,
    /// `com.apple.security.cs.allow-dyld-environment-variables` is granted,
    /// without which `DYLD_INSERT_LIBRARIES` is ignored.
    pub allows_dyld_env: bool,
    /// `com.apple.security.cs.disable-library-validation` is granted, without
    /// which the ad-hoc signed doorstop dylib is refused.
    pub disables_library_validation: bool,
    /// Signed ad hoc (no identity), which is what a stripped signature looks like.
    pub adhoc: bool,
}

/// A hardened-runtime signature needs all three injection entitlements: without
/// unsigned executable memory the game is killed once Mono starts JIT-ing, and
/// without dyld environment variables or with library validation doorstop is
/// never loaded at all. Missing any of them, the signature has to go.
pub fn needs_resign(status: &SigningStatus) -> bool {
    status.hardened_runtime
        && !(status.allows_unsigned_exec_memory
            && status.allows_dyld_env
            && status.disables_library_validation)
}

/// Asks `codesign` about `executable` and reads off what matters for injection.
/// An unsigned executable has no hardened runtime, so it reports as such rather
/// than as an error.
pub fn signing_status(executable: &Path) -> Result<SigningStatus> {
    let output = Command::new("codesign")
        .args(["-dv", "--entitlements", ":-"])
        .arg(executable)
        .output()
        .context("failed to run codesign")?;

    // the code directory goes to stderr and the entitlements to stdout
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push('\n');
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        if text.contains("not signed at all") {
            warn!(path = %executable.display(), "game executable is not signed");
            return Ok(SigningStatus::default());
        }

        bail!(
            "codesign failed to read the signature of {} ({}): {}",
            executable.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(parse_signing_status(&text))
}

/// Parses the combined stdout and stderr of `codesign -dv --entitlements :-`.
fn parse_signing_status(output: &str) -> SigningStatus {
    let mut status = SigningStatus::default();

    for line in output.lines() {
        let line = line.trim();

        // e.g. `CodeDirectory v=20500 size=489 flags=0x10000(runtime) hashes=4+7 location=embedded`;
        // only the flags token counts, `Runtime Version=` is a different line
        if line.starts_with("CodeDirectory")
            && let Some(flags) = line
                .split_whitespace()
                .find_map(|token| token.strip_prefix("flags="))
        {
            status.hardened_runtime |= flags.contains("runtime");
            status.adhoc |= flags.contains("adhoc");
        }

        if line == "Signature=adhoc" {
            status.adhoc = true;
        }
    }

    status.allows_unsigned_exec_memory =
        entitlement_is_true(output, ENTITLEMENT_UNSIGNED_EXEC_MEMORY);
    status.allows_dyld_env = entitlement_is_true(output, ENTITLEMENT_DYLD_ENV);
    status.disables_library_validation =
        entitlement_is_true(output, ENTITLEMENT_DISABLE_LIBRARY_VALIDATION);

    status
}

/// Whether the entitlements plist in `output` grants `key` (`<key>…</key>`
/// followed by `<true/>`).
fn entitlement_is_true(output: &str, key: &str) -> bool {
    let key_tag = format!("<key>{key}</key>");

    output.match_indices(&key_tag).any(|(index, _)| {
        let value = output[index + key_tag.len()..].trim_start();
        value.starts_with("<true/>") || value.starts_with("<true>")
    })
}

/// Replaces the signature of `app_bundle` (and everything inside it) with an
/// ad-hoc one, which drops the hardened runtime and its entitlements, so that
/// doorstop can inject into the game.
///
/// This is the same thing r2modmac does on install. It is reversible: Steam's
/// "Verify integrity of game files" restores the shipped, signed binaries.
pub fn strip_hardened_runtime(app_bundle: &Path) -> Result<()> {
    info!(
        bundle = %app_bundle.display(),
        "re-signing game ad hoc to remove the hardened runtime"
    );

    let output = Command::new("codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(app_bundle)
        .output()
        .context("failed to run codesign")?;

    if !output.status.success() {
        bail!(
            "{}",
            resign_failure_message(
                app_bundle,
                &output.status.to_string(),
                &String::from_utf8_lossy(&output.stderr)
            )
        );
    }

    info!(bundle = %app_bundle.display(), "re-signed game ad hoc");

    Ok(())
}

/// The user-facing error for a failed `codesign`. Modifying another app's bundle
/// needs the App Management permission (macOS 13+); without it codesign fails
/// with "Operation not permitted", which needs a pointer rather than raw output.
fn resign_failure_message(app_bundle: &Path, status: &str, stderr: &str) -> String {
    let stderr = stderr.trim();

    if stderr.contains("Operation not permitted") {
        format!(
            "macOS blocked Gale from re-signing {}: grant Gale \"App Management\" in \
             System Settings > Privacy & Security > App Management, then launch again",
            app_bundle.display()
        )
    } else {
        format!(
            "codesign failed to re-sign {} ({}): {}",
            app_bundle.display(),
            status,
            stderr
        )
    }
}

#[cfg(test)]
mod resign_message_tests {
    use super::*;

    #[test]
    fn permission_denied_points_at_app_management() {
        let msg = resign_failure_message(
            Path::new("/Games/Valheim.app"),
            "exit status: 1",
            "/Games/Valheim.app: replacing existing signature\n/Games/Valheim.app: Operation not permitted\nIn subcomponent: /Games/Valheim.app/Contents/PlugIns/X.bundle",
        );
        assert!(msg.contains("App Management"), "{msg}");
        assert!(msg.starts_with("macOS blocked Gale from re-signing /Games/Valheim.app"));
    }

    #[test]
    fn other_failures_keep_codesign_output() {
        let msg = resign_failure_message(Path::new("/Games/X.app"), "exit status: 1", "  bad things  ");
        assert_eq!(msg, "codesign failed to re-sign /Games/X.app (exit status: 1): bad things");
    }
}

/// The `.app` bundle that `executable` lives in, if any.
fn app_bundle_of(executable: &Path) -> Option<&Path> {
    executable.ancestors().find(|path| {
        path.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    })
}

/// Makes sure doorstop can inject into the game at `game_dir`, re-signing it if
/// its signature would get it killed and the user has allowed that (once: after
/// a successful re-sign the signature is ad hoc and passes the check). Games
/// that are not app bundles are left alone.
fn ensure_injectable(game_name: &str, game_dir: &Path, allow_resign: bool) -> Result<()> {
    let Some(executable) = super::find_app_bundle_executable(game_dir) else {
        debug!(
            game_dir = %game_dir.display(),
            "no app bundle in game directory, skipping signature check"
        );
        return Ok(());
    };

    let status = signing_status(&executable)?;
    debug!(?status, path = %executable.display(), "game signature");

    if !needs_resign(&status) {
        return Ok(());
    }

    if !allow_resign {
        bail!(
            "{game_name} is signed with Apple's hardened runtime, which prevents BepInEx from \
             loading (macOS kills the game and reports it as 'damaged'). Enable 'Allow \
             re-signing the game' in Settings to let Gale re-sign it (reversible with \
             Steam's Verify integrity)."
        );
    }

    let bundle = app_bundle_of(&executable)
        .ok_or_else(|| eyre!("{} is not inside an app bundle", executable.display()))?;

    strip_hardened_runtime(bundle)?;

    let status = signing_status(&executable)?;
    ensure!(
        !needs_resign(&status),
        "re-signing {} did not remove the hardened runtime ({:?})",
        bundle.display(),
        status
    );

    Ok(())
}

fn extract_dylib(zip_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))?;
    let entry = archive
        .by_name(DOORSTOP_ZIP_ENTRY)
        .with_context(|| format!("{DOORSTOP_ZIP_ENTRY} not found in archive"))?;

    // the declared size comes straight from the (unverified) archive, so it
    // must not drive an allocation; the real dylib is ~100 KiB
    ensure!(
        entry.size() <= MAX_DYLIB_SIZE,
        "{DOORSTOP_ZIP_ENTRY} is unexpectedly large ({} bytes)",
        entry.size()
    );

    let mut dylib = Vec::new();
    entry.take(MAX_DYLIB_SIZE + 1).read_to_end(&mut dylib)?;
    ensure!(
        dylib.len() as u64 <= MAX_DYLIB_SIZE,
        "{DOORSTOP_ZIP_ENTRY} is unexpectedly large"
    );

    Ok(dylib)
}

fn verify_hash(dylib: &[u8]) -> Result<()> {
    let actual = blake3::hash(dylib).to_hex();

    ensure!(
        actual.as_str() == DOORSTOP_BLAKE3,
        "doorstop dylib hash mismatch: expected {DOORSTOP_BLAKE3}, got {actual}. \
         The download may be corrupted or tampered with; try again later."
    );

    Ok(())
}

/// Writes `contents` to a fresh file at `path` atomically: the data goes to a
/// uniquely named sibling temp file first and is then renamed over the
/// destination, so a crash or full disk mid-write never leaves a truncated
/// file behind (Steam runs the launcher without Gale, so a half-written dylib
/// would break launches until Gale next runs). The rename also replaces the
/// directory entry rather than modifying the existing file, because files that
/// a package installed are hard links into the mod cache and must not be
/// modified in place.
///
/// The temp name is unique per call rather than a fixed `<file>.tmp` because
/// two writers can race on the same profile: a BepInExPack install kicks off a
/// background doorstop download, and a launch pressed right after downloads it
/// too. Each writer renames its own complete file into place and the loser's
/// rename never fails on a temp file the winner already moved.
fn write_replacing(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let file_name = path
        .file_name()
        .ok_or_else(|| eyre!("{} has no file name", path.display()))?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    let mut prefix = file_name.to_owned();
    prefix.push(".");

    let mut tmp = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(".tmp")
        // what `fs::write` would create, before the umask
        .permissions(PermissionsExt::from_mode(0o666))
        .tempfile_in(parent)
        .fs_context("creating temporary file", parent)?;

    // dropping `tmp` on any error below removes the temp file
    tmp.write_all(contents.as_ref())
        .fs_context("writing file", tmp.path())?;
    tmp.persist(path)
        .map_err(|err| err.error)
        .fs_context("replacing file", path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tempfile::tempdir;

    use super::*;

    /// Only runs with `--ignored` and `GALE_DOORSTOP_DYLIB` pointing at a local
    /// copy of `universal/libdoorstop.dylib` from the 4.5.0 release zip.
    #[test]
    #[ignore]
    fn hash_pin_matches_release_dylib() {
        let path = std::env::var("GALE_DOORSTOP_DYLIB")
            .expect("set GALE_DOORSTOP_DYLIB to the release libdoorstop.dylib");
        let bytes = fs::read(path).unwrap();
        assert_eq!(blake3::hash(&bytes).to_hex().as_str(), DOORSTOP_BLAKE3);
        verify_hash(&bytes).unwrap();
    }

    fn crc32(data: &[u8]) -> u32 {
        !data.iter().fold(!0u32, |crc, byte| {
            (0..8).fold(crc ^ u32::from(*byte), |c, _| {
                if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }
            })
        })
    }

    /// A stored-entry zip whose only entry is `universal/libdoorstop.dylib`
    /// holding `data`, with a ZIP64 extra field claiming `declared_size`
    /// uncompressed bytes.
    fn zip_with_declared_size(data: &[u8], declared_size: u64) -> Vec<u8> {
        let name = DOORSTOP_ZIP_ENTRY.as_bytes();
        let crc = crc32(data);
        let mut extra = Vec::new();
        extra.extend_from_slice(&0x0001u16.to_le_bytes());
        extra.extend_from_slice(&16u16.to_le_bytes());
        extra.extend_from_slice(&declared_size.to_le_bytes());
        extra.extend_from_slice(&(data.len() as u64).to_le_bytes());

        let mut out = Vec::new();
        // local file header
        out.extend_from_slice(&0x04034b50u32.to_le_bytes());
        out.extend_from_slice(&45u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&[0; 4]); // time, date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&u32::MAX.to_le_bytes()); // compressed -> zip64
        out.extend_from_slice(&u32::MAX.to_le_bytes()); // uncompressed -> zip64
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&extra);
        out.extend_from_slice(data);

        let cd_offset = out.len() as u32;
        // central directory header
        out.extend_from_slice(&0x02014b50u32.to_le_bytes());
        out.extend_from_slice(&45u16.to_le_bytes()); // version made by
        out.extend_from_slice(&45u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&[0; 4]); // time, date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&u32::MAX.to_le_bytes());
        out.extend_from_slice(&u32::MAX.to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // comment len
        out.extend_from_slice(&0u16.to_le_bytes()); // disk start
        out.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        out.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        out.extend_from_slice(&0u32.to_le_bytes()); // local header offset
        out.extend_from_slice(name);
        out.extend_from_slice(&extra);
        let cd_size = out.len() as u32 - cd_offset;

        // end of central directory
        out.extend_from_slice(&0x06054b50u32.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // disk
        out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
        out.extend_from_slice(&1u16.to_le_bytes()); // entries on disk
        out.extend_from_slice(&1u16.to_le_bytes()); // entries total
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // comment len
        out
    }

    #[test]
    fn extract_dylib_reads_zip64_entry() {
        let data = b"not really a dylib";
        let zip = zip_with_declared_size(data, data.len() as u64);

        assert_eq!(extract_dylib(&zip).unwrap(), data);
    }

    #[test]
    fn extract_dylib_rejects_huge_declared_size_without_allocating() {
        // 64 TiB: on this host `Vec::with_capacity` of this size aborts the
        // process rather than panicking
        let zip = zip_with_declared_size(b"tiny", 1 << 46);

        let err = extract_dylib(&zip).unwrap_err();
        assert!(err.to_string().contains("unexpectedly large"), "{err}");
    }

    #[test]
    fn hash_mismatch_is_rejected() {
        let err = verify_hash(b"not the doorstop").unwrap_err();
        assert!(err.to_string().contains("hash mismatch"), "{err}");
    }

    #[test]
    fn write_replacing_breaks_hard_links() {
        let dir = tempdir().unwrap();
        let original = dir.path().join("cache");
        let linked = dir.path().join("profile");
        fs::write(&original, "4.4.0").unwrap();
        fs::hard_link(&original, &linked).unwrap();

        write_replacing(&linked, "4.5.0").unwrap();

        assert_eq!(fs::read_to_string(&original).unwrap(), "4.4.0");
        assert_eq!(fs::read_to_string(&linked).unwrap(), "4.5.0");
    }

    #[test]
    fn write_replacing_leaves_no_temp_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(DYLIB_NAME);

        write_replacing(&path, "first").unwrap();
        write_replacing(&path, "second").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from(DYLIB_NAME)]);
    }

    #[test]
    fn write_replacing_creates_readable_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(DYLIB_NAME);

        write_replacing(&path, "dylib").unwrap();

        // the game loads the dylib, so it gets what `fs::write` would give it
        // (0666 before the umask), not a temp file's private 0600
        let plain = dir.path().join("plain");
        fs::write(&plain, "dylib").unwrap();
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&path), mode(&plain));
    }

    /// Two writers racing on the same path (a background post-install download
    /// and a launch pressed right after) must both succeed: each renames its
    /// own complete file into place, and nothing is left behind.
    #[test]
    fn write_replacing_survives_concurrent_writers() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(DYLIB_NAME);
        let contents = vec![0xABu8; 128 * 1024];

        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| write_replacing(&path, &contents)))
                .collect();
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
        });

        assert_eq!(fs::read(&path).unwrap(), contents);
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from(DYLIB_NAME)]);
    }

    /// A stale `<file>.tmp` from an older Gale (or anything else) is neither
    /// used nor removed: the temp name is unique per call.
    #[test]
    fn write_replacing_ignores_fixed_tmp_sibling() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(DYLIB_NAME);
        let stale = dir.path().join(format!("{DYLIB_NAME}.tmp"));
        fs::write(&stale, "stale").unwrap();

        write_replacing(&path, "fresh").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "fresh");
        assert_eq!(fs::read_to_string(&stale).unwrap(), "stale");
    }

    fn fake_game(game_dir: &Path) -> PathBuf {
        let bundle = game_dir.join("Foo.app");
        let contents = bundle.join("Contents");
        fs::create_dir_all(contents.join("MacOS")).unwrap();

        fs::write(
            contents.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>CFBundleExecutable</key><string>Foo</string></dict></plist>"#,
        )
        .unwrap();

        // the "game" reports what the launcher set up for it
        let exe = contents.join("MacOS").join("Foo");
        fs::write(
            &exe,
            "#!/bin/sh\nprintf '%s\\n' \"$DOORSTOP_ENABLED\" \"$DOORSTOP_TARGET_ASSEMBLY\" \"$PWD\" \"$*\"\n",
        )
        .unwrap();
        fs::set_permissions(&exe, PermissionsExt::from_mode(0o755)).unwrap();

        bundle
    }

    fn fake_profile() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        fs::write(dylib_path(dir.path()), b"").unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/core")).unwrap();
        fs::write(dir.path().join("BepInEx/core/BepInEx.Preloader.dll"), b"").unwrap();
        ensure_launcher(dir.path()).unwrap();
        dir
    }

    fn run_launcher(profile_dir: &Path, target: &Path, args: &[&str]) -> Vec<String> {
        let output = Command::new(launcher_path(profile_dir))
            .arg(target)
            .args(args)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "launcher failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn launcher_resolves_bundle_and_uses_gale_args() {
        let profile = fake_profile();
        let game_dir = tempdir().unwrap();
        let bundle = fake_game(game_dir.path());
        let custom_target = profile.path().join("BepInEx/core/BepInEx.Preloader.dll");

        // what Steam passes with `/bin/sh "run_bepinex.sh" %command%`: the bundle, then
        // Gale's doorstop arguments, then the user's own launch arguments
        let lines = run_launcher(
            profile.path(),
            &bundle,
            &[
                "--doorstop-enabled",
                "true",
                "--doorstop-target-assembly",
                custom_target.to_str().unwrap(),
                "-console",
            ],
        );

        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], custom_target.to_str().unwrap());
        assert_eq!(
            fs::canonicalize(&lines[2]).unwrap(),
            fs::canonicalize(game_dir.path()).unwrap()
        );
        assert_eq!(lines[3], "-console");
    }

    #[test]
    fn launcher_accepts_bundle_executable_directly() {
        let profile = fake_profile();
        let game_dir = tempdir().unwrap();
        let exe = fake_game(game_dir.path()).join("Contents/MacOS/Foo");

        let lines = run_launcher(profile.path(), &exe, &[]);

        assert_eq!(lines[0], "1");
        // the launcher resolves its own directory physically (`pwd -P`)
        assert_eq!(
            fs::canonicalize(&lines[1]).unwrap(),
            fs::canonicalize(profile.path().join("BepInEx/core/BepInEx.Preloader.dll")).unwrap()
        );
        assert_eq!(
            fs::canonicalize(&lines[2]).unwrap(),
            fs::canonicalize(game_dir.path()).unwrap()
        );
        assert_eq!(lines[3], "");
    }

    #[test]
    fn launcher_fails_without_doorstop() {
        let profile = tempdir().unwrap();
        ensure_launcher(profile.path()).unwrap();
        let game_dir = tempdir().unwrap();
        let bundle = fake_game(game_dir.path());

        let output = Command::new(launcher_path(profile.path()))
            .arg(&bundle)
            .output()
            .unwrap();

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("libdoorstop.dylib"));
    }

    #[test]
    fn launcher_is_written_executable() {
        let dir = tempdir().unwrap();

        ensure_launcher(dir.path()).unwrap();

        let path = launcher_path(dir.path());
        let script = fs::read_to_string(&path).unwrap();
        assert_eq!(script.lines().next(), Some("#!/bin/sh"));
        assert_eq!(script, LAUNCHER_SCRIPT);

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }
}

/// Signature parsing against captured `codesign -dv --entitlements :-` output.
/// Nothing here runs codesign.
#[cfg(test)]
mod signing_tests {
    use super::*;

    const ENTITLEMENTS_HEAD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>com.apple.security.cs.allow-dyld-environment-variables</key>
	<true/>
	<key>com.apple.security.cs.allow-jit</key>
	<true/>
	<key>com.apple.security.cs.disable-library-validation</key>
	<true/>
"#;

    const ENTITLEMENTS_TAIL: &str = "</dict>\n</plist>\n";

    /// What Valheim ships with: Developer ID, hardened runtime, no
    /// unsigned-executable-memory entitlement.
    const SHIPPED_DIRECTORY: &str = "\
Executable=/Users/me/Library/Application Support/Steam/steamapps/common/Valheim/Valheim.app/Contents/MacOS/Valheim
Identifier=com.irongate.valheim
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20500 size=489 flags=0x10000(runtime) hashes=4+7 location=embedded
Signature size=9046
Timestamp=12 Aug 2026 at 10:15:03
Info.plist entries=26
TeamIdentifier=ABCDE12345
Runtime Version=13.3.0
Sealed Resources version=2 rules=13 files=6
Internal requirements count=1 size=180
";

    /// The same game after `codesign --force --deep --sign -`.
    const ADHOC_DIRECTORY: &str = "\
Executable=/Users/me/Library/Application Support/Steam/steamapps/common/Valheim/Valheim.app/Contents/MacOS/Valheim
Identifier=com.irongate.valheim
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20400 size=489 flags=0x2(adhoc) hashes=4+7 location=embedded
Signature=adhoc
Info.plist entries=26
TeamIdentifier=not set
Sealed Resources version=2 rules=13 files=6
Internal requirements count=0 size=12
";

    fn shipped_output() -> String {
        format!("{ENTITLEMENTS_HEAD}{ENTITLEMENTS_TAIL}\n{SHIPPED_DIRECTORY}")
    }

    #[test]
    fn shipped_hardened_runtime_needs_resign() {
        let status = parse_signing_status(&shipped_output());

        assert_eq!(
            status,
            SigningStatus {
                hardened_runtime: true,
                allows_unsigned_exec_memory: false,
                allows_dyld_env: true,
                disables_library_validation: true,
                adhoc: false,
            }
        );
        assert!(needs_resign(&status));
    }

    #[test]
    fn hardened_runtime_with_unsigned_exec_memory_is_fine() {
        let output = format!(
            "{ENTITLEMENTS_HEAD}\t<key>{ENTITLEMENT_UNSIGNED_EXEC_MEMORY}</key>\n\t<true/>\n{ENTITLEMENTS_TAIL}\n{SHIPPED_DIRECTORY}"
        );

        let status = parse_signing_status(&output);

        assert!(status.hardened_runtime);
        assert!(status.allows_unsigned_exec_memory);
        assert!(status.allows_dyld_env);
        assert!(status.disables_library_validation);
        assert!(!needs_resign(&status));
    }

    /// Entitlements plist granting only `allow-jit` plus the given keys.
    fn entitlements_with(keys: &[&str]) -> String {
        let mut plist = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n\t<key>com.apple.security.cs.allow-jit</key>\n\t<true/>\n",
        );
        for key in keys {
            plist.push_str(&format!("\t<key>{key}</key>\n\t<true/>\n"));
        }
        plist.push_str(ENTITLEMENTS_TAIL);
        plist
    }

    #[test]
    fn hardened_runtime_without_dyld_env_needs_resign() {
        // DYLD_INSERT_LIBRARIES is ignored, so doorstop never loads
        let output = format!(
            "{}\n{SHIPPED_DIRECTORY}",
            entitlements_with(&[
                ENTITLEMENT_UNSIGNED_EXEC_MEMORY,
                ENTITLEMENT_DISABLE_LIBRARY_VALIDATION,
            ])
        );

        let status = parse_signing_status(&output);

        assert!(status.hardened_runtime);
        assert!(status.allows_unsigned_exec_memory);
        assert!(!status.allows_dyld_env);
        assert!(status.disables_library_validation);
        assert!(needs_resign(&status));
    }

    #[test]
    fn hardened_runtime_with_library_validation_needs_resign() {
        // the ad-hoc signed doorstop dylib is refused by library validation
        let output = format!(
            "{}\n{SHIPPED_DIRECTORY}",
            entitlements_with(&[ENTITLEMENT_UNSIGNED_EXEC_MEMORY, ENTITLEMENT_DYLD_ENV])
        );

        let status = parse_signing_status(&output);

        assert!(status.hardened_runtime);
        assert!(status.allows_unsigned_exec_memory);
        assert!(status.allows_dyld_env);
        assert!(!status.disables_library_validation);
        assert!(needs_resign(&status));
    }

    #[test]
    fn all_three_entitlements_via_helper_are_fine() {
        let output = format!(
            "{}\n{SHIPPED_DIRECTORY}",
            entitlements_with(&[
                ENTITLEMENT_UNSIGNED_EXEC_MEMORY,
                ENTITLEMENT_DYLD_ENV,
                ENTITLEMENT_DISABLE_LIBRARY_VALIDATION,
            ])
        );

        assert!(!needs_resign(&parse_signing_status(&output)));
    }

    #[test]
    fn entitlements_alone_do_not_matter_without_hardened_runtime() {
        // a Developer ID signature without the runtime flag lets dyld inject freely
        let output = format!(
            "{}\nCodeDirectory v=20400 size=489 flags=0x0(none) hashes=4+7 location=embedded\nTeamIdentifier=ABCDE12345\n",
            entitlements_with(&[])
        );

        let status = parse_signing_status(&output);

        assert!(!status.hardened_runtime);
        assert!(!status.allows_dyld_env);
        assert!(!needs_resign(&status));
    }

    #[test]
    fn entitlement_set_to_false_does_not_count() {
        let output = format!(
            "{ENTITLEMENTS_HEAD}\t<key>{ENTITLEMENT_UNSIGNED_EXEC_MEMORY}</key>\n\t<false/>\n{ENTITLEMENTS_TAIL}\n{SHIPPED_DIRECTORY}"
        );

        let status = parse_signing_status(&output);

        assert!(!status.allows_unsigned_exec_memory);
        assert!(needs_resign(&status));
    }

    #[test]
    fn adhoc_signature_after_stripping_passes() {
        // an ad-hoc re-sign drops the entitlements, so stdout is empty
        let status = parse_signing_status(&format!("\n{ADHOC_DIRECTORY}"));

        assert_eq!(
            status,
            SigningStatus {
                hardened_runtime: false,
                allows_unsigned_exec_memory: false,
                allows_dyld_env: false,
                disables_library_validation: false,
                adhoc: true,
            }
        );
        assert!(!needs_resign(&status));
    }

    #[test]
    fn adhoc_is_read_from_flags_alone() {
        let status = parse_signing_status(
            "CodeDirectory v=20400 size=300 flags=0x20002(adhoc,linker-signed) hashes=4+2 location=embedded\n",
        );

        assert!(status.adhoc);
        assert!(!status.hardened_runtime);
    }

    #[test]
    fn adhoc_with_runtime_flag_still_needs_resign() {
        let status = parse_signing_status(
            "CodeDirectory v=20500 size=489 flags=0x10002(adhoc,runtime) hashes=4+7 location=embedded\nSignature=adhoc\n",
        );

        assert!(status.adhoc);
        assert!(status.hardened_runtime);
        assert!(needs_resign(&status));
    }

    #[test]
    fn runtime_version_line_is_not_the_hardened_runtime() {
        let status = parse_signing_status(
            "CodeDirectory v=20400 size=489 flags=0x0(none) hashes=4+7 location=embedded\nRuntime Version=13.3.0\n",
        );

        assert!(!status.hardened_runtime);
        assert!(!needs_resign(&status));
    }

    #[test]
    fn unsigned_output_reports_nothing() {
        let status = parse_signing_status(
            "/path/Foo.app/Contents/MacOS/Foo: code object is not signed at all\n",
        );

        assert_eq!(status, SigningStatus::default());
        assert!(!needs_resign(&status));
    }

    #[test]
    fn app_bundle_of_walks_up_to_the_bundle() {
        let exe = Path::new("/Games/Valheim/Valheim.app/Contents/MacOS/Valheim");

        assert_eq!(
            app_bundle_of(exe),
            Some(Path::new("/Games/Valheim/Valheim.app"))
        );
        assert_eq!(app_bundle_of(Path::new("/Games/Foo/start_game.sh")), None);
    }

    #[test]
    fn doorstop_installed_requires_dylib_and_matching_version() {
        let dir = tempfile::tempdir().unwrap();

        assert!(!doorstop_installed(dir.path()));

        fs::write(dylib_path(dir.path()), b"").unwrap();
        assert!(!doorstop_installed(dir.path()));

        fs::write(version_path(dir.path()), "0.0.1\n").unwrap();
        assert!(!doorstop_installed(dir.path()));

        fs::write(version_path(dir.path()), format!("{DOORSTOP_VERSION}\n")).unwrap();
        assert!(doorstop_installed(dir.path()));
    }
}
