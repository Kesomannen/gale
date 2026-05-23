#!/usr/bin/env bash
flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install flatpak-build com.kesomannen.gale.yml
