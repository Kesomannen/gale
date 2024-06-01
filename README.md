![logo](https://raw.githubusercontent.com/Kesomannen/gale/master/app-icon@0,25x.png)

# Gale Mod Manager

A lightweight and fast mod manager for Thunderstore, built with [SvelteKit](https://kit.svelte.dev/) and [Tauri](https://tauri.app/).

## Features

- Support for most of the games on Thunderstore, including Lethal Company, Risk of Rain 2 and Content Warning
- A performant and responsive UI with a tiny download and disk size (10 MB!)
- Import & export profiles (compatible with r2modman/thunderstore mod manager)
- Feature rich mod config editor
- Launch games through Steam *or* directly, with any number of instances
- Usable with the "Install with Mod Manager" button on Thunderstore
- Automatically transfer profiles from other mod managers
- Automatic app updates
- Local mod imports

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

### On Windows

Make sure you have done all of the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) and installed the tauri CLI with `cargo install tauri-cli`.
After you have cloned the repository, run
```sh
npm install
cargo tauri dev
```
to start a dev server. Alternatively, do `cargo tauri build` to build an installer for your platform. If you want to modify/distribute the app, keep in mind the [license](https://choosealicense.com/licenses/gpl-3.0/#).

### On Linux

***Install the required tools and build chain***

Notice: The examples use a Debian flavor package manager. Use your system specific package manager such as Yum/Pacman instead`

To build Gale on Linux you will likely require the following packages:

***Rust Dependencies***

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

***Cargo/Rust***

```sh
sudo apt install cargo
```

***NVM***

If your system does not have the latest node available you can install nvm by following the instructions [here](https://www.freecodecamp.org/news/node-version-manager-nvm-install-guide/)

You can install a version of node that will work with Gale by using

```sh
nvm install 20
```

Then you set the node version for your current terminal by using this command:

```sh
nvm use 20
```

You will need to set your node version before using any NPM commands later in this guide.

***Install TauriCLI***

```sh
cargo install tauri-cli
```

This will take a while, especially if you've never compiled anything with rust before.

Congrats, you're now ready to actually start building.

***Build Gale***

```sh
nvm use 20
npm install
cargo tauri build
```

This will take a decent chunk of time if its your first time compiling Gale. There will be some ignorable warnings but it will output both an installable package (a .deb for debian flavor systems) and an executable (eg: a bundled appimage) under `src-tauri/target/release/bundle`

## Screenshots

*Browse mods*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot2.png)

*Profile*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot1.png)

*Config editor*

![screenshot](https://raw.githubusercontent.com/Kesomannen/gale/master/images/screenshot3.png)

## Credits

Material icons licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html).

Logo font based on [Poppins](https://fonts.google.com/specimen/Poppins).

## License

[GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/#)
