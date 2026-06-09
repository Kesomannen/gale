#!/usr/bin/env bash
flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install flatpak-build flatpak/com.kesomannen.gale.dev.yml
