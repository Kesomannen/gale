![Gale](https://raw.githubusercontent.com/Kesomannen/gale/master/images/banner.png)

[![Thunderstore Version](https://img.shields.io/thunderstore/v/Kesomannen/GaleModManager?style=flat)](https://thunderstore.io/c/lethal-company/p/Kesomannen/GaleModManager/)
[![Discord](https://img.shields.io/discord/1288196347597688912?style=flat&label=discord)](https://discord.gg/sfuWXRfeTt)
[![Website](https://img.shields.io/website?url=https%3A%2F%2Fkesomannen.com%2Fgale&up_message=online&down_message=offline&style=flat)](https://kesomannen.com/gale)
[![GitHub License](https://img.shields.io/github/license/Kesomannen/gale?style=flat)](https://github.com/Kesomannen/gale?tab=GPL-3.0-1-ov-file#readme)

The modern and lightweight mod manager for [Thunderstore](https://thunderstore.io), built with [Svelte](https://kit.svelte.dev/) and [Tauri](https://tauri.app/).

## Features

- Support for all 150+ games on Thunderstore, including Lethal Company, R.E.P.O and Risk Of Rain 2
- An intuitive and responsive interface
- Tiny download size and resource usage
- Feature-rich mod config editor
- Automatic profile syncing (beta)
- AI-powered translation for mod names, descriptions, and config entries (NEW)

### AI Translation

Gale supports AI-powered translation for mod names, descriptions, and configuration entries. This feature uses any OpenAI-compatible API (e.g., OpenAI, DeepSeek, Moonshot, etc.).

**How to use:**
1. Go to **Settings** → **AI Translation**
2. Enable the feature and configure your API endpoint
3. Enter your API URL and API Key
4. Adjust batch size if needed (default: 20 mods per request)
5. Use the translate button in the toolbar or mod details page

**Features:**
- Batch translation (translate multiple mods in one API call)
- Translation cache (persistent storage in local database)
- Chinese/Japanese/Korean search support (search mods by translated names)
- Toggle switch to show/hide translations
- Auto-translation when browsing mod list
- Config entry translation

[...and more](https://github.com/Kesomannen/gale/wiki/Features)

## Installation

### Windows

#### Manual (Thunderstore)

- Go to the [Thunderstore page](https://thunderstore.io/c/lethal-company/p/Kesomannen/GaleModManager/) and click _Manual Download_.
- Extract the downloaded .zip file (for example by right-clicking and choosing _Extract All_).
- Run the `Gale_X.X.X_x64_en-US.msi` file inside of the extracted folder.

#### Manual (Github)

- Go to [Releases](https://github.com/Kesomannen/gale/releases).
- Download the `Gale_X.X.X_x64_en-US.msi` file for your desired version (the latest is recommended).
- Run the downloaded file.

> [!NOTE]
> You might get a prompt saying "Windows has protected your PC". In this case, click `More Info` and `Run Anyway`.

> [!TIP]
> If you're unsure about the safety of this app, I would suggest running it through a service like [VirusTotal](https://www.virustotal.com).

#### Scoop

Gale is available as a [Scoop](https://scoop.sh/) app:

```powershell
scoop install https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/gale.json
```

To update, run:

```powershell
scoop uninstall gale
scoop install https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/gale.json
```

### Linux

#### Arch Linux

Gale is available as a community-maintained [AUR package](https://aur.archlinux.org/packages/gale-bin).

Example installation command:

```bash
yay -S gale-bin
```

#### Debian

Gale is available as a .deb package in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, install with:

```bash
sudo dpkg -i Gale_X.X.X_x64_en-US.deb
```

#### Fedora

Gale is available as a .rpm package in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, install with:

```bash
sudo rpm -i Gale_X.X.X_x64_en-US.rpm
```

#### AppImage

Distribution-agnostic AppImages are available in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, make the file executable and run it:

```bash
chmod +x Gale_X.X.X_x64_en-US.AppImage
./Gale_X.X.X_x64_en-US.AppImage
```

---

Want to build it from source? See the [wiki](https://github.com/Kesomannen/gale/wiki/building-from-source).

## Screenshots

_Profile_

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshots/screenshot1.png)

_Thunderstore browser_

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshots/screenshot2.png)

_Mod config editor_

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshots/screenshot3.png)

_Modpack export_

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshots/screenshot4.png)

## Credits

Material icons licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html).

Thanks to Ebkr for helping to navigate the thunderstore API and BepInEx, and of course making the original mod manager!

---

Still have questions? See the [FAQ](https://github.com/Kesomannen/gale/wiki/faq) or a [detailed list of features](https://github.com/Kesomannen/gale/wiki/Features).
