#!/bin/sh
# Gale's macOS launcher for BepInEx (UnityDoorstop 4). Gale rewrites this file
# on every launch, so do not edit it. To launch through Steam, set the game's
# launch options to:   /bin/sh "<path to this script>" %command%
# (Steam on macOS refuses to exec a script directly, so name the interpreter.)
#
# GALE_MACOS_ARCH picks the slice of a universal game to run on Apple Silicon:
# `auto` (the default), `native` or `x86_64`. See pick_arch below.
set -u

BASEDIR=$(cd "$(dirname "$0")" && pwd -P)
log() { printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*" >> "$BASEDIR/gale_launch.log"; }
die() { log "error: $*"; echo "gale: $*" >&2; exit 1; }

[ $# -ge 1 ] || die "usage: $0 <game .app or executable> [args...]"
exe=${1%/}
shift

# Steam substitutes the .app bundle for %command%; resolve the binary inside it.
case "$exe" in
    *.app)
        name=$(defaults read "$exe/Contents/Info" CFBundleExecutable 2>/dev/null) \
            || die "cannot read CFBundleExecutable from $exe"
        exe="$exe/Contents/MacOS/$name"
        ;;
esac
[ -x "$exe" ] || die "game executable not found: $exe"
game_dir=$(dirname "${exe%.app/Contents/MacOS/*}")

doorstop="$BASEDIR/libdoorstop.dylib"
target=
enabled=1

# Gale passes the doorstop settings as arguments; use them and keep them away
# from the game. Every other argument is passed through in its original order.
i=$#
while [ $i -gt 0 ]; do
    case "$1" in
        --doorstop-enabled|--doorstop-target-assembly)
            [ $i -ge 2 ] || die "missing value for $1"
            case "$1" in
                --doorstop-enabled)
                    case "$2" in 1|true|TRUE|t|T|y|Y|yes) enabled=1 ;; *) enabled=0 ;; esac ;;
                *) target=$2 ;;
            esac
            shift; i=$((i - 1))
            ;;
        *) set -- "$@" "$1" ;;
    esac
    shift; i=$((i - 1))
done

# A vanilla launch (Gale passes `--doorstop-enabled false`) needs neither the
# doorstop nor BepInEx, so only require and inject them when doorstop is on.
if [ "$enabled" = 1 ]; then
    [ -f "$doorstop" ] || die "missing $doorstop (Gale downloads it when launching)"
    # Without Gale's arguments (Play pressed in Steam itself), pick the preloader
    # the same way Gale does: the first of these that exists in BepInEx/core.
    if [ -z "$target" ]; then
        core="$BASEDIR/BepInEx/core"
        for name in BepInEx.Unity.Mono.Preloader.dll BepInEx.Unity.IL2CPP.dll \
                    BepInEx.Preloader.dll BepInEx.IL2CPP.dll; do
            if [ -f "$core/$name" ]; then target="$core/$name"; break; fi
        done
        [ -n "$target" ] || die "no BepInEx preloader in $core (is BepInEx installed in this profile?)"
    fi

    [ -f "$target" ] || die "missing $target (is BepInEx installed in this profile?)"

    export DOORSTOP_ENABLED="$enabled"
    export DOORSTOP_TARGET_ASSEMBLY="$target"
    export DOORSTOP_IGNORE_DISABLED_ENV=0
    export DOORSTOP_MONO_DLL_SEARCH_PATH_OVERRIDE=""
    # Prepend, so anything already injected (such as the Steam overlay) keeps loading.
    export DYLD_LIBRARY_PATH="$BASEDIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
    export DYLD_INSERT_LIBRARIES="$doorstop${DYLD_INSERT_LIBRARIES:+:$DYLD_INSERT_LIBRARIES}"
fi

# UnityDoorstop cannot hook the Mono runtime of Unity 6 on arm64
# (NeighTools/UnityDoorstop#108), so a modded Unity 6 Mono game has to run its
# x86_64 slice under Rosetta. Everything else runs natively.
pick_arch() {
    case "${GALE_MACOS_ARCH:-auto}" in
        native|arm64) echo native; return ;;
        x86_64|rosetta) echo x86_64; return ;;
    esac
    [ "$enabled" = 1 ] || { echo native; return; }
    contents=${exe%/MacOS/*}
    [ -f "$contents/Frameworks/libmonobdwgc-2.0.dylib" ] || { echo native; return; }
    info=$(defaults read "$contents/Info" CFBundleGetInfoString 2>/dev/null) || info=
    major=$(printf '%s\n' "$info" | sed -n 's/.*Unity Player version \([0-9][0-9]*\)\..*/\1/p' | head -n 1)
    if [ -n "$major" ] && [ "$major" -ge 6000 ]; then echo x86_64; else echo native; fi
}

if [ "$(sysctl -n hw.optional.arm64 2>/dev/null)" = 1 ]; then
    arch=$(pick_arch)
    log "exec $exe (cwd $game_dir) enabled=$enabled arch=$arch target=$target args=$*"
    cd "$game_dir" || die "cannot enter $game_dir"

    if [ "$arch" = x86_64 ]; then
        arch -x86_64 /usr/bin/true 2>/dev/null \
            || die "this game needs Rosetta 2 to run with mods. Install it with: softwareupdate --install-rosetta --agree-to-license"
        export ARCHPREFERENCE=x86_64
        flag=-x86_64
    else
        # Run the native slice even when the parent process (Steam) is x86_64.
        export ARCHPREFERENCE=arm64,x86_64
        flag=
    fi

    # arch is an arm64e system binary, so DYLD_* in its own environment would be
    # injected into arch itself (NeighTools/UnityDoorstop#107). Hand them to the
    # game through -e only.
    set -- "$exe" "$@"
    [ -n "${DYLD_LIBRARY_PATH:-}" ] && set -- -e "DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH" "$@"
    [ -n "${DYLD_INSERT_LIBRARIES:-}" ] && set -- -e "DYLD_INSERT_LIBRARIES=$DYLD_INSERT_LIBRARIES" "$@"
    unset DYLD_LIBRARY_PATH DYLD_INSERT_LIBRARIES
    exec arch $flag "$@"
else
    log "exec $exe (cwd $game_dir) enabled=$enabled target=$target args=$*"
    cd "$game_dir" || die "cannot enter $game_dir"
    exec "$exe" "$@"
fi
