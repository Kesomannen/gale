![Gale](https://raw.githubusercontent.com/Kesomannen/gale/master/images/banner.png)

[![Thunderstore Version](https://img.shields.io/thunderstore/v/Kesomannen/GaleModManager?style=flat)](https://thunderstore.io/c/lethal-company/p/Kesomannen/GaleModManager/)
[![Discord](https://img.shields.io/discord/1288196347597688912?style=flat&label=discord)](https://discord.gg/sfuWXRfeTt)
[![GitHub License](https://img.shields.io/github/license/Kesomannen/gale?style=flat)](https://github.com/Kesomannen/gale?tab=GPL-3.0-1-ov-file#readme)

A modern and lightweight mod manager for [Thunderstore](https://thunderstore.io), built with [Svelte](https://kit.svelte.dev/) and [Tauri](https://tauri.app/).

## Features

- Support for all 150+ games on Thunderstore, including Lethal Company, R.E.P.O and Risk Of Rain 2
- An intuitive and responsive interface
- Tiny download size and resource usage
- Feature-rich mod config editor
- Automatic profile syncing

[...and more](https://github.com/Kesomannen/gale/wiki/Features)

## Installation

### Windows

<details>
  <summary>
    <b>Manual (Thunderstore)</b>
  </summary>
  
  - Go to the [Thunderstore page](https://thunderstore.io/c/lethal-company/p/Kesomannen/GaleModManager/) and click _Manual Download_.
  - Extract the downloaded .zip file (for example by right-clicking and choosing _Extract All_).
  - Run the `Gale_X.X.X_x64_en-US.msi` file inside of the extracted folder.
</details>

<details>
  <summary>
    <b>Manual (Github)</b>
  </summary>
  
  - Go to [Releases](https://github.com/Kesomannen/gale/releases).
  - Download the `Gale_X.X.X_x64_en-US.msi` file for your desired version (the latest is recommended).
  - Run the downloaded file.
</details>

<details>
  <summary>
    <b>Scoop</b>
  </summary>
  
  Gale is available as an independently hosted [Scoop](https://scoop.sh/) app:

  ```powershell
  scoop install https://raw.githubusercontent.com/Kesomannen/scoop-games/refs/heads/master/bucket/gale.json
  ```
  
  To update, run:
  
  ```powershell
  scoop uninstall gale
  scoop install https://raw.githubusercontent.com/Kesomannen/scoop-games/refs/heads/master/bucket/gale.json
  ```

  > Efforts are in progress to submit the app to the [Scoop games bucket](https://github.com/Calinou/scoop-games/pull/1756) to make installation and updating more straightforward.
</details>

> [!NOTE]
> You might get a prompt saying "Windows has protected your PC". In this case, click `More Info` and `Run Anyway`.

> [!TIP]
> If you're unsure about the safety of this app, I would suggest running it through a service like [VirusTotal](https://www.virustotal.com).

### Linux

<details>
  <summary>
    <b>Arch</b>
  </summary>
  
  Gale is available as a **community-maintained** [AUR package](https://aur.archlinux.org/packages/gale-bin).
  
  Example installation command:
  
  ```bash
  yay -S gale-bin
  ```
</details>


<details>
  <summary>
    <b>Debian</b>
  </summary>

  Gale is available as a .deb package in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, install with:

  ```bash
  sudo dpkg -i Gale_X.X.X_x64_en-US.deb
  ```
</details>

<details>
  <summary>
    <b>Fedora</b>
  </summary>

  Gale is available as a .rpm package in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, install with:

  ```bash
  sudo rpm -i Gale_X.X.X_x64_en-US.rpm
  ```
</details>

<details>
  <summary>
    <b>Flatpak</b>
  </summary>

  Gale is available as an independently hosted Flatpak package:

  ```bash
  flatpak install https://kesomannen.com/com.kesomannen.gale.flatpakref
  ```
</details>

<details>
  <summary>
    <b>AppImage</b>
  </summary>

  Distribution-agnostic AppImages are available in [Releases](https://github.com/Kesomannen/gale/releases). After downloading, make the file executable and run it:
  
  ```bash
  chmod +x Gale_X.X.X_x64_en-US.AppImage
  ./Gale_X.X.X_x64_en-US.AppImage
  ```
</details>

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
