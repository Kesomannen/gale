# Gale Native macOS Design

## Goal

Build Gale as a native, unsigned Apple Silicon macOS application using its
existing Tauri and Svelte codebase. The accepted milestone displays Gale's
complete interface and supports browsing Thunderstore and managing profiles.

Game discovery and game launching are explicitly outside this milestone.

## Architecture

The existing frontend and Rust application remain the single source of truth.
Tauri uses WKWebView on macOS, avoiding the unsupported WebView2 rendering path
encountered through Wine.

Platform-specific Rust code is selected at compile time:

- Existing Windows and Linux behavior remains unchanged.
- Cross-platform browsing, downloads, configuration, database, and profile
  management compile and run on macOS.
- Operations that require a supported game-launch platform return a clear
  unsupported-platform error on macOS.

No parallel macOS shell or duplicated frontend is introduced.

## Dependencies and Build

Cargo dependencies gain the macOS features required by their upstream crates,
particularly native keychain support. Windows- and Linux-only dependencies and
APIs remain target-gated.

The existing Tauri configuration supplies the macOS icon and application
metadata. Local builds produce an unsigned `.app` bundle for Apple Silicon.
Code signing, notarization, universal binaries, and public distribution are
outside this milestone.

## User Experience

The native application opens a normal macOS window containing Gale's full
interface. Browsing packages, selecting games, editing profiles, and other
platform-neutral features behave as they do on existing platforms.

If the user reaches a game discovery or launch action, Gale reports that the
operation is not supported on macOS instead of panicking, hanging, or invoking
Windows/Linux commands.

## Error Handling

Compile-time platform boundaries prevent unsupported APIs from entering the
macOS binary. Runtime boundaries use explicit errors for intentionally
unsupported game integration. Existing application logging remains available
for native startup and frontend failures.

## Testing and Acceptance

Automated validation covers:

- frontend type checking and linting;
- Rust formatting, checks, and tests on macOS;
- a release-mode native Tauri build;
- preservation of Windows and Linux compilation through existing CI.

The definitive smoke test launches the generated `.app` on this Apple Silicon
Mac and verifies:

1. the main window renders visible Gale content;
2. Thunderstore browsing loads;
3. profile creation and selection work;
4. attempting an unsupported game operation produces an actionable error.

The failed Wine/Homebrew wrapper is removed from the deliverable and replaced
with native macOS build and installation documentation.
