{
  pkgs,
  src,
}: let
  inherit (pkgs) lib;
  inlangModules = [
    {
      url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js";
      src = pkgs.fetchurl {
        url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4.4.4/dist/index.js";
        hash = "sha256-siz2DrKLPIw84ftjAGEaBVLxLQ2ZXTfE3SyW462AxkU=";
      };
    }
    {
      url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-m-function-matcher@2/dist/index.js";
      src = pkgs.fetchurl {
        url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-m-function-matcher@2.2.14/dist/index.js";
        hash = "sha256-hYYvYwV5O1a/2a/lNosJbmP7Kuqzi3eZwFFRe+NJnAs=";
      };
    }
  ];
in
  pkgs.rustPlatform.buildRustPackage (finalAttrs: {
    pname = "gale";
    version = (builtins.fromJSON (builtins.readFile ../../package.json)).version;
    inherit src;

    cargoRoot = "src-tauri";
    buildAndTestSubdir = finalAttrs.cargoRoot;
    cargoLock.lockFile = ../../src-tauri/Cargo.lock;

    pnpmDeps = pkgs.fetchPnpmDeps {
      inherit (finalAttrs) pname version;
      src = lib.fileset.toSource {
        root = ../..;
        fileset = lib.fileset.unions [../../package.json ../../pnpm-lock.yaml];
      };
      pnpm = pkgs.pnpm_11;
      fetcherVersion = 4;
      hash = "sha256-ouZevE5lDhzUGtVDifHd92zdeEvR1d8SrkzlmsUijBc=";
    };

    nativeBuildInputs = with pkgs;
      [
        nodejs_24
        pnpm_11
        pnpmConfigHook
        cargo-tauri.hook
        pkg-config
        cmake
        python3
        jq
        moreutils
      ]
      ++ lib.optionals stdenv.hostPlatform.isLinux [wrapGAppsHook3]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [makeWrapper];

    buildInputs = with pkgs;
      lib.optionals stdenv.hostPlatform.isLinux [
        gtk3
        webkitgtk_4_1
        glib-networking
        libsoup_3
        libayatana-appindicator
        librsvg
        openssl
        dbus
        udev
      ];

    postPatch = ''
      jq '.bundle.createUpdaterArtifacts = false' src-tauri/tauri.conf.json \
        | sponge src-tauri/tauri.conf.json
      substituteInPlace project.inlang/settings.json ${
        lib.concatMapStringsSep " " (module: "--replace-fail ${module.url} ${module.src}") inlangModules
      }
    '';

    postInstall = lib.optionalString pkgs.stdenv.hostPlatform.isDarwin ''
      mkdir -p "$out/bin"
      # Register the bundle when launched from the CLI as well as from Finder.
      # macOS reads URL schemes from Info.plist; the plugin cannot register them.
      makeWrapper "$out/Applications/Gale.app/Contents/MacOS/gale" "$out/bin/gale" \
        --run "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f \"$out/Applications/Gale.app\""
    '';

    postFixup = lib.optionalString pkgs.stdenv.hostPlatform.isDarwin ''
      # Sign the complete bundle after stripping, not just its Mach-O executable.
      /usr/bin/codesign --force --sign - "$out/Applications/Gale.app"
    '';

    meta = {
      description = "A lightweight mod manager for Thunderstore";
      homepage = "https://github.com/Kesomannen/gale";
      license = lib.licenses.gpl3Only;
      mainProgram = "gale";
      platforms = lib.platforms.linux ++ lib.platforms.darwin;
    };
  })
