# Changelog

## Unreleased

### Fixed

- Welcome dialog opening every time the app is launched until a setting is changed

## 0.6.1 (2024-07-03)

### Added

- Preview of contents before importing a profile

### Changed

- Display a list of mod cards instead of table when viewing Dependants/Dependencies

### Fixed

- BepInEx/* directories in remote mods being installed at the wrong location
- Settings on the welcome page not working properly

## 0.6.0 (2024-07-02)

### Added

- Revamped modpack export page, including uploading modpacks directly from Gale

### Changed

- **Breaking change!** Corrected the community identifier for Risk of Rain 2. If you use Gale for RoR2, please go to Gale's data folder and rename the `ror2` directory to `riskofrain2`!
- Persist mod search options between sessions
- Allow importing local DLLs directly
- UI changes

### Fixed

- Filters not updating specifically for Risk of Rain 2
- Trim whitespace around import codes and search queries
- Allow enabling mods with missing dependencies
- BepInEx/* directories in local mods being installed at the wrong location

## 0.5.8 (2024-06-26)

### Fixed

- Frequent crashing while mods are being fetched from thunderstore
- Optimize game icons to reduce binary size
- Mods and settings loading slowly
- The window opening extremely small

## 0.5.7 (2024-06-14)

### Added

- Ability to rename and duplicate the active profile from the menubar
- Setting to override a game's install path

### Changed

- Various UI changes

### Fixed

- Improve error handling during startup, instead of crashing immediately
- Some mod files being skipped when importing a profile with config
- Config editor clearing search query when deleting a file

## 0.5.6 (2024-06-10)

### Added

- Setting for the steam library path

### Fixed

- Improve game search accuracy
- Correct default steam library path on Linux

## 0.5.5 (2024-06-09)

### Added

- Option to disable mod download cache
- Mod reordering (AKA custom sorting)
- Disk space sorting option

### Changed

- Show latest changelog instead of the mod's version's
- Moved settings to a separate page instead of a dialog
- Made mod updates cancellable
- Show only dependants from the active profile
- Miscellaneous UI changes

### Fixed

- Mod details menu not refreshing after installing or updating a mod
- No mods being loaded if BepInEx hadn't been installed through another mod manager before
- Config files being sorted by raw file name instead of display name
- Disabled mods being enabled after updating
- Mod dependencies not being installed when updating
- Failing to extract mods in rare cases

## 0.5.4 (2024-06-04)

### Fixed

- Config files being imported outside of the BepInEx directory

## 0.5.3 (2024-06-04)

### Added

- Ability to copy dependency strings to clipboard
- Button to view dependants in mod details menu dropdown

### Changed

- Remember window size and position between sessions
- Show cleaner file names in mod config editor
- Several user interface alterations

### Fixed

- Profile dropdown in the import profile dialog allowing invalid options
- Import profile dialog selecting the wrong option initially
- Config files outside of the `config` directory not being exported/imported
- Update banner sometimes not showing until the app is restarted
- Config files not being linked until the config editor page is visited
- Improve config file linking

## 0.5.2 (2024-06-02)

### Added

- README to mod details menu, shown if the window is large enough
- "Edit config" button to mod details menu
- Quick install button in mod list
- "Open directory" options to profile mod dropdown

### Changed

- Update banner can now be dismissed
- Save search queries, filters and sorting options are between page navigations
- Various UI improvements

### Fixed

- Local mod icons not being displayed
- Links replacing the entire page instead of opening in the browser
- Crash when importing local mod
- Profile names being capitalized in the import profile dialog
- R2modman/TMM profiles not being found on Linux
- Improve performance of config serialization
- Config editor sometimes freezing when switching between profiles

## 0.5.1 (2024-05-31)

### Added

- "Change version" option to profile mod dropdown
- Icon in the top right to indicate if an app update is available
- Disable and enable all mods button
- Open Gale log button

### Changed

- Minor UI tweaks
- Moved settings to Edit menu

### Fixed

- Moving data directory
- Moving cache directory before any mods are installed

## 0.5.0 (2024-05-30)

### Added

- Setup flow for first-time users
- Cache for mods that are used in profiles, which drastically decreases load times
- Option to automatically transfer profiles from r2modman/Thunderstore Mod Manager
- Soft cache clear (only removes unused mods)
- Dialog when installing a mod with missing dependencies (instead of throwing an error)

### Changed

- Changelog on the home page now hides unreleased changes
- Overhauled sorting and filtering options
- Infinite scroll instead of pagination
- Made msi the preferred installer for Windows
- Numerous UI changes

### Fixed

- Moving the window while a dialog is open
- Improve startup time by not parsing config files immediately
- "steam_exe_path pref not found" error when opening settings
- Config sliders with large ranges causing performance issues or freezing
- Zoom factor not being applied when reopening the app
- Open log file button
- Modpack export creating an incorrect PNG

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
