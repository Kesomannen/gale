#!/usr/bin/env bash
flatpak-builder --force-clean --install-deps-from=flathub --repo=repo $@ build com.kesomannen.gale.dev.yml
