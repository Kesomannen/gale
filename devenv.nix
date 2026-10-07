{
  pkgs,
  lib,
  ...
}: {
  languages.javascript = {
    enable = true;
    package = pkgs.nodejs_24;
    pnpm.enable = true;
    pnpm.package = pkgs.pnpm_11;
  };

  languages.rust.enable = true;

  packages = with pkgs;
    [
      cargo-tauri
      # Native Node modules and Rust dependencies need a C compiler and build tools.
      python3
      cmake
      pkg-config
    ]
    ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
      gtk3
      webkitgtk_4_1
      libayatana-appindicator
      librsvg
      openssl
      dbus
      udev
      patchelf
    ];

  env = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
    LD_LIBRARY_PATH = lib.makeLibraryPath (with pkgs; [
      gtk3
      webkitgtk_4_1
      libayatana-appindicator
      librsvg
      udev
    ]);
  };

  tasks."gale:check-tools" = {
    before = ["devenv:enterTest"];
    exec = ''
      set -euo pipefail
      node --version
      pnpm --version
      rustc --version
      cargo --version
      cargo tauri --version
      cargo clippy --version
      rustfmt --version
      ${lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
        pkg-config --exists gtk+-3.0 webkit2gtk-4.1 openssl dbus-1 libudev
      ''}
    '';
  };
}
