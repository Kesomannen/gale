![logo](https://raw.githubusercontent.com/Kesomannen/gale/master/app-icon@0,25x.png)

<h1> Gale Mod Manager </h1>

A lightweight mod manager for Thunderstore, built with [SvelteKit](https://kit.svelte.dev/) and [Tauri](https://tauri.app/).

<h2> Table of Contents </h2>

- [Features](#features)
- [Installation](#installation)
  - [Windows](#windows)
  - [MacOS](#macos)
  - [Linux](#linux)
- [Building from source](#building-from-source)
  - [Windows](#on-windows)
  - [Linux](#on-linux)
- [Screenshots](#screenshots)
- [Credits](#credits)
- [License](#license)


## Features

- Support for most of the games on Thunderstore, including Lethal Company, Risk of Rain 2 and Content Warning
- A performant and responsive UI with a tiny download and disk size (10 MB!)
- Import & export profiles (compatible with r2modman/thunderstore mod manager)
- Feature rich mod config editor
- Export modpacks automatically, including uploading directly to Thunderstore
- Launch games through Steam *or* directly, with any number of instances
- Usable with the "Install with Mod Manager" button on Thunderstore
- Automatically transfer profiles from other mod managers
- Automatic app updates
- Local mod imports
- CLI (see the [wiki page](https://github.com/Kesomannen/gale/wiki/CLI/))

**Coming soon**

- Support for modding frameworks other than BepInEx

## Installation

### Windows

**Through Thunderstore**
- Go to the [thunderstore page](https://thunderstore.io/c/lethal-company/p/Kesomannen/GaleModManager/) and press "Manual Download".
- Unzip the downloaded .zip file (for example by right-clicking and choosing "Extract All").
- Run the `Gale_X.X.X_x64_en-US.msi` file inside of the unzipped folder.

**Through Github**
- Go to [Releases](https://github.com/Kesomannen/gale/releases).
- Download the `Gale_X.X.X_x64_en-US.msi` file for your desired version (the latest is recommended).
- Run the downloaded file.

> **Note:** You might get a prompt saying "Windows has protected your PC". In this case, click `More Info` and `Run Anyway`.

> If you're unsure about the safety of this app, I would suggest running it through a service like [VirusTotal](https://www.virustotal.com).
> It's also worth noting that this project is fully open-source, which means any developer could look at the code and easily spot any malware.

### MacOS

TBD

### Linux

TBD

## Building from source

### Windows

Firstly, make sure you have done all of the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites).

Additionally, make sure you have git and the tauri CLI installed (or install it with `cargo install tauri-cli`).

After you have cloned the repository, run the following to start a dev server:
```sh
npm install
cargo tauri dev
```
To build Gale, run:
```sh
cargo tauri build
```
After a while, it will output both an .msi and an .exe installer in the `src-tauri/target/release/bundle` folder.

> If you want to modify/distribute the app, keep in mind the [license](/LICENSE).

### Linux

**Install the required tools and build chain**

> **Note:** The examples use a Debian flavor package manager. Use your system specific package manager such as Yum or Pacman instead.

To build Gale on Linux you will likely require the following packages:

**Rust Dependencies**

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Cargo/Rust**

```sh
sudo apt install cargo
```

**NVM**

If your system does not have the latest node available you can install nvm by following the instructions [here](https://www.freecodecamp.org/news/node-version-manager-nvm-install-guide/).

You can install a version of node that will work with Gale by using:

```sh
nvm install 20
```

Then you set the node version for your current terminal by using this command:

```sh
nvm use 20
```

You will need to set your node version before using any NPM commands later in this guide.

**Install the Tauri CLI**

```sh
cargo install tauri-cli
```

This will take a while, especially if you've never compiled anything with Rust before.

**Start the dev server**

```sh
nvm use 20
npm install
cargo tauri dev
```

**Build Gale**

```sh
cargo tauri build
```
After a while, this it will output both an executable and an installable package (a .deb for debian-based systems) in the `src-tauri/target/release/bundle` directory.

> If you want to modify/distribute the app, keep in mind the [license](/LICENSE).

## Screenshots

*Mod list*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot3.png)

*Profile*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot2.png)

*Config editor*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot1.png)

*Modpack export*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot1.png)

## Credits

Material icons licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html)

## License

[GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/#)
