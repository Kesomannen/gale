#!/bin/sh

# You might need to restart your pc if sharun doesn't create `AppDir` in this directory (It should create dirs on its own)

set -eu

# Always build in this script's directory regardless of the caller's cwd
cd "$(dirname "$0")"
unset APPDIR

ARCH="$(uname -m)"
SHARUN="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/main/useful-tools/quick-sharun.sh"
DEBLOATED_PKGS="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/get-debloated-pkgs.sh"

# You might need to comment UPINFO if building locally
export UPINFO="gh-releases-zsync|${GITHUB_REPOSITORY%/*}|${GITHUB_REPOSITORY#*/}|latest|*$ARCH.AppImage.zsync"
export OUTNAME=gale-anylinux-"$ARCH".AppImage
export DESKTOP=$(echo ../src-tauri/target/release/bundle/deb/Gale_*_amd64/data/usr/share/applications/Gale.desktop)
export ICON=$(echo ../src-tauri/target/release/bundle/deb/Gale_*_amd64/data/usr/share/icons/hicolor/256x256@2/apps/gale.png)

export OUTPATH=./dist

#Remove leftovers
rm -rf AppDir dist appinfo

# ADD LIBRARIES
wget --retry-connrefused --tries=30 "$SHARUN" -O ./quick-sharun
wget --retry-connrefused --tries=30 "$DEBLOATED_PKGS" -O ./get-debloated-pkgs
chmod +x ./quick-sharun ./get-debloated-pkgs

./get-debloated-pkgs --add-mesa --add-vulkan

# Point to binaries
./quick-sharun \
        ../src-tauri/target/release/bundle/deb/Gale_*_amd64/data/usr/bin/gale

# Make AppImage
./quick-sharun --make-appimage

echo "All Done!"
