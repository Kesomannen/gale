#!/bin/sh
set -eu

script=''
architecture=''
remaining=$#
while [ "$remaining" -gt 0 ]; do
    case "$1" in
        --gale-bepinex)
            [ "$remaining" -ge 3 ] && [ -z "$script" ] || exit 1
            script=$2
            architecture=$3
            shift 3
            remaining=$((remaining - 3))
            ;;
        *)
            set -- "$@" "$1"
            shift
            remaining=$((remaining - 1))
            ;;
    esac
done

if [ -z "$script" ]; then
    exec "$@"
fi

case "$architecture" in
    x86_64|arm64) ;;
    *) exit 1 ;;
esac

# Enter the loader's architecture before its script sets DYLD variables:
# passing those variables through arch or another system executable can lose them.
exec /usr/bin/arch "-$architecture" /bin/sh "$script" "$@"
