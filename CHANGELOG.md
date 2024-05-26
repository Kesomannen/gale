# Changelog

## Unreleased

### Added

- Soft cache clear (only removes unused mods)
- Dialog when installing a mod with missing dependencies (instead of throwing an error)

### Changed

- Infinite scroll instead of pagination
- Made msi the preferred installer for Windows
- Renamed "Preferences" to "Settings"
- Numerous UI changes

### Fixed

- Zoom factor not being applied when reopening the app
- Open log file button
- Modpack icon resizing outputing an incorrect PNG

## 0.4.1 (2024-05-23)

### Added

- Log file
- Dialog when overwriting a profile

### Removed

- Deep link functionality on Linux

### Changed

- Moved "New Profile" to the profile list, instead of the menubar
- Various UI improvements

### Fixed

- Crash when launching on Linux
- Deserializing config files with commas as decimal separators
- Multiple versions of the same mod being installed as dependencies

## 0.4.0 (2024-05-22)

### Added

- Dialog when enabling a mod which has disabled dependencies
- Dialog before updating all mods
- More options when importing profiles (similarly to r2modman/Thunderstore Mod Manager)
- Close button to all dialogs
- Confirm dialog when aborting mod installation
- Zoom preference

### Removed

- Quit button

### Changed

- Config parser now allows invalid semver versions
- Increased interval between fetching mods from Thunderstore
- Various UI improvements

### Fixed

- Auto updater not working (*not retroactive; you still need to manually update to this version*)
- Improve performance for dependency trees
- Improve performance of config parsing
- Parsing config entries without a value
- "Update all" banner showing the wrong number of mods
- Config files being copied from other profiles when importing from code
- Launching games with doorstop v4
- Uninstalled mods sometimes not being deleted from the file system

## 0.3.1 (2024-05-15)

### Fixed

- Compatibility with profiles created prior to 0.3.0

## 0.3.0 (2024-05-15)

### Added

- Profile import from file
- Profile export to file
- Ability to cancel mod installation
- Ability to remove mod without its dependencies
- Mod disabling and enabling

### Changed

- The config parser now supports untagged entries
- Decimal config sliders now have a step of 0.01 (instead of 1)
- Various UI improvements

### Fixed

- Importing profiles with disabled mods

## 0.2.0 (2024-05-08)

### Added 

- Proper logo & icons

### Changed

- Gale itself is now hidden in the mod list
- Config entries are no longer required to be in the acceptable range

### Fixed 

- Crash when opening on Linux (hopefully) (thanks testaccount666 on discord)
- Screenshots in the Thunderstore README

## 0.1.0 (2024-05-07)

- Initial release
