# Dedicated servers

Gale can launch a dedicated server installed on the same computer or deploy the active profile to a remote server. Dedicated server support is shown only for games with a `dedicatedServer` entry in `src-tauri/games.json`.

## Using it

Open the dedicated server dialog from the arrow beside the profile launch button.

### Local server

Choose **Local server**, enter the server name, world, port, and optional password, then select **Launch server**. Gale locates the game's dedicated server through its configured platform, prepares the mod loader, and starts the server with the active profile.

The server must already be installed. Gale manages only the process it starts; closing Gale does not silently terminate an unrelated server process.

### Remote server

Choose **Remote server** and enter the connection details supplied by the server host.

- **SFTP (SSH)** supports password and private-key authentication. Confirm the host-key fingerprint the first time Gale connects.
- **FTP / FTPS (automatic)** attempts encrypted explicit FTPS first. It falls back to plain FTP only when the host does not support TLS. Self-signed certificates require confirmation before credentials are sent.
- **Server directory** is the directory exposed by the host. Use `/` when the FTP account already opens at the server root.

Use **Test connection** before deploying. **Deploy profile** first shows a preview of uploads, removals, unchanged files, and client-only mods that will be skipped.

## What gets synchronized

Gale mirrors these directories from the active profile:

- `BepInEx/plugins`
- `BepInEx/patchers`
- `BepInEx/config`

A mirrored directory is made to match the profile. Files missing from the server are uploaded, changed files are replaced, and server files absent from the profile are removed. Disabling a mod produces `.old` files locally; those disabled files are not uploaded and their enabled server counterparts are removed. Empty remote mod directories are cleaned up when possible.

Gale skips generated BepInEx data, logs, Thunderstore package metadata (`README.md`, `CHANGELOG.md`, `icon.png`, and `manifest.json`), and packages marked client-only by Thunderstore or Hexium.

If the host already provides BepInEx, Gale preserves its core and startup files. This matters for managed hosts where those files are installed through the control panel or cannot be modified. Hosts that expose `plugins`, `patchers`, and `config` directly at the account root are detected automatically.

The preview is based on both the current remote listing and Gale's previous deployment manifest. Manually deleted server files are uploaded again even when the local profile has not changed. Files outside the mirrored directories are removed only when Gale previously deployed them.

Cleanup failures do not discard a successful upload. Gale continues with other removals and reports items the host refused to delete. Connection or upload failures still stop the deployment because the profile cannot be considered synchronized.

## Maintainer notes

The backend lives in `src-tauri/src/profile/server`:

- `commands.rs` exposes Tauri commands and coordinates settings, secrets, progress events, and process state.
- `local.rs` locates and launches local dedicated server installations.
- `deploy.rs` filters the profile, reads the remote tree, builds the preview, and applies the deployment plan.
- `remote.rs` provides the common SFTP and FTP/FTPS operations.
- `manifest.rs` defines `.gale-server-manifest.json` and validates stored relative paths.
- `runtime.rs` tracks the local server process.

Server settings are stored per profile. Passwords and private-key passphrases use the operating system credential store rather than the profile database.

Remote deployment uses BLAKE3 hashes and file sizes to avoid uploading unchanged files. SFTP replacements use temporary and backup names for an atomic rename where supported. FTP uploads reconnect and retry transient transfer failures, but FTP servers do not provide the same atomic replacement guarantees.

The manifest records files from the last successful deployment. In a normal layout it is stored at the selected server directory; in a restricted root layout it is stored under `config`. Do not expand deletion beyond the three mirrored directories without preserving the manifest path validation and preview behavior.

Frontend API bindings are in `src/lib/api/profile/server.ts`, shared state is in `src/lib/state/server.svelte.ts`, and the dialog components are in `src/lib/components/dialogs`. User-facing text belongs in `messages/en.json` and must be accessed through Paraglide.

When adding another supported game, define its dedicated-server platforms and default port in `src-tauri/games.json`. Local launching also depends on Gale being able to locate that platform's dedicated-server installation and executable.
