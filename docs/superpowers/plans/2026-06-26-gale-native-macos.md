# Gale Native macOS Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and launch Gale as a native unsigned Apple Silicon macOS application with visible UI and platform-neutral functionality.

**Architecture:** Keep the existing Tauri/Svelte application and introduce only compile-time macOS boundaries where code currently assumes Linux or Windows. WKWebView supplies native rendering; unsupported game integration returns explicit errors while browsing and profile management remain available.

**Tech Stack:** Rust 1.87+, Cargo, Node.js 24, pnpm 11, Svelte 5, Tauri 2, WKWebView, GitHub Actions macOS runners.

---

## File Structure

- Modify `src-tauri/Cargo.toml`: enable macOS-compatible dependency features.
- Modify `src-tauri/src/profile/launch/custom_args.rs`: treat macOS command-line parsing as Unix.
- Modify `src-tauri/src/profile/launch/mod.rs`: define non-Proton behavior and reject launches on macOS.
- Modify `src-tauri/src/profile/launch/platform.rs`: compile platform launch helpers on macOS and report unsupported operations.
- Modify `src-tauri/src/lib.rs`: show native startup errors on macOS.
- Modify `.github/workflows/check.yaml`: compile and test the native macOS target.
- Modify `.github/workflows/publish.yaml`: build a macOS application artifact.
- Modify `README.md`: replace Wine instructions with native build/install status.
- Delete `Formula/gale-wine.rb` and `packaging/macos/*`: remove the failed Wine delivery path.

### Task 1: Establish the Native Build Baseline

**Files:**
- No repository changes

- [ ] **Step 1: Install the required local toolchains**

Run:

```sh
brew install node@24 pnpm rustup-init
rustup-init -y --default-toolchain stable
rustup toolchain install 1.87.0
rustup default 1.87.0
```

Expected: `node --version`, `pnpm --version`, and `cargo --version` succeed.

- [ ] **Step 2: Install project dependencies**

Run:

```sh
pnpm install --frozen-lockfile
```

Expected: exit 0 and no lockfile changes.

- [ ] **Step 3: Capture the failing macOS compile errors**

Run:

```sh
pnpm check
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: frontend check succeeds; Cargo reports the exact Linux/Windows assumptions requiring macOS guards.

### Task 2: Make Cross-Platform Core Compile on macOS

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/profile/launch/custom_args.rs`
- Modify: `src-tauri/src/profile/launch/mod.rs`
- Modify: `src-tauri/src/profile/launch/platform.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add a failing macOS launch-support unit test**

Add to the test module in `src-tauri/src/profile/launch/custom_args.rs`:

```rust
#[test]
fn unix_quoted_args() {
    let result = CustomArgs::from_str(r#"--name "Mac Profile""#).unwrap();
    assert_eq!(result.args, vec!["--name", "Mac Profile"]);
}
```

Run:

```sh
cargo test --manifest-path src-tauri/Cargo.toml unix_quoted_args
```

Expected: compilation fails because `join` and `split` have no macOS branch.

- [ ] **Step 2: Make Unix argument handling include macOS**

Change both Linux guards in `custom_args.rs` to:

```rust
#[cfg(unix)]
```

Change both Windows guards to:

```rust
#[cfg(not(unix))]
```

Run the focused test again. Expected: the test progresses past `custom_args`.

- [ ] **Step 3: Enable macOS keychain support**

Replace the `keyring` feature list in `Cargo.toml` with:

```toml
keyring = { version = "3", features = [
    "apple-native",
    "windows-native",
    "linux-native-sync-persistent",
] }
```

- [ ] **Step 4: Add explicit macOS launch errors**

At the start of `ManagedGame::launch` in `profile/launch/mod.rs`, add:

```rust
#[cfg(target_os = "macos")]
bail!("game discovery and launching are not yet supported on macOS");
```

Define the non-Linux Proton value in `launch_command` as:

```rust
#[cfg(not(target_os = "linux"))]
let is_proton = false;
```

In `platform.rs`, add:

```rust
#[cfg(target_os = "macos")]
fn create_base_steam_command() -> Result<Command> {
    bail!("Steam launching is not yet supported on macOS")
}
```

- [ ] **Step 5: Include macOS in native startup dialogs**

Change the setup-error dialog guard in `lib.rs` to:

```rust
#[cfg(any(target_os = "windows", target_os = "macos"))]
```

- [ ] **Step 6: Iterate on compiler-reported platform boundaries**

Run:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
```

For each remaining error, add the narrowest `cfg` branch that either reuses Unix behavior or returns:

```rust
bail!("this operation is not yet supported on macOS")
```

Do not add game discovery or launching behavior.

- [ ] **Step 7: Run Rust verification**

Run:

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: all exit 0.

- [ ] **Step 8: Commit native compilation**

```sh
git add src-tauri
git commit -m "feat: compile Gale natively on macOS"
```

### Task 3: Build and Smoke-Test the Native Application

**Files:**
- Modify only if smoke testing exposes a defect.

- [ ] **Step 1: Run frontend verification**

Run:

```sh
pnpm check
pnpm lint
```

Expected: both exit 0.

- [ ] **Step 2: Build the native application**

Run:

```sh
pnpm tauri build --bundles app
```

Expected: `src-tauri/target/release/bundle/macos/Gale.app` exists.

- [ ] **Step 3: Launch the application**

Run:

```sh
open src-tauri/target/release/bundle/macos/Gale.app
```

Expected: a native Gale window displays its full interface.

- [ ] **Step 4: Exercise accepted functionality**

Verify manually:

1. Thunderstore packages render.
2. A profile can be created and selected.
3. Settings remain usable.
4. A game launch action reports that launching is unsupported on macOS.

- [ ] **Step 5: Run a second launch**

Quit and reopen the `.app`. Expected: stored state loads and the interface remains visible.

### Task 4: Replace Wine Packaging with Native CI and Documentation

**Files:**
- Delete: `Formula/gale-wine.rb`
- Delete: `packaging/macos/gale-wine`
- Delete: `packaging/macos/test-gale-wine.sh`
- Delete: `packaging/macos/check-formula-launcher.sh`
- Modify: `.github/workflows/check.yaml`
- Modify: `.github/workflows/publish.yaml`
- Modify: `README.md`

- [ ] **Step 1: Remove the Wine packaging files and validation job**

Delete the files above and remove `validate-gale-wine` from `publish.yaml`.

- [ ] **Step 2: Add macOS validation**

Add a `macos-14` entry to the existing check workflow matrix, installing Node,
pnpm, and Rust exactly as the other jobs do. Run:

```yaml
- run: cargo test --manifest-path src-tauri/Cargo.toml
- run: pnpm tauri build --bundles app
```

- [ ] **Step 3: Add macOS to release builds**

Add this matrix entry to `publish.yaml`:

```yaml
- platform: macos-14
  args: '--bundles app'
```

- [ ] **Step 4: Replace README Wine instructions**

Document that the current macOS artifact is an unsigned Apple Silicon native
build, how to open it locally, and that game discovery/launching remain
unsupported.

- [ ] **Step 5: Verify repository checks**

Run:

```sh
pnpm check
pnpm lint
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
pnpm tauri build --bundles app
git diff --check
```

Expected: every command exits 0.

- [ ] **Step 6: Commit packaging and documentation**

```sh
git add .github README.md Formula packaging
git commit -m "ci: build native Gale app for macOS"
```

### Task 5: Final Acceptance

**Files:**
- No repository changes unless verification exposes a defect.

- [ ] **Step 1: Launch the freshly rebuilt bundle**

```sh
open src-tauri/target/release/bundle/macos/Gale.app
```

- [ ] **Step 2: Confirm acceptance criteria**

Confirm visible UI, Thunderstore browsing, profile management, persistent
second launch, and actionable unsupported game-launch behavior.

- [ ] **Step 3: Record final status**

Run:

```sh
git status --short
git log --oneline -5
```

Report the bundle path, verification results, and remaining macOS limitations.
