# Changelog

## Unreleased

### Added

- Confirmation dialog for config file deletion

### Fixed

- Made the mod details menu scrollable when the description overflows
- Some packages not being fetched from small communities

## 1.6.1 (2025-05-20)

### Changed

- Profile imports now ignore missing mods instead of failing

### Fixed

- Profile sync status not being refreshed on launch
- Some config fields being shown as a color picker instead of a number input

## 1.6.0 (2025-05-14)

### Added

- Beta profile sync feature
  - Read more on [the wiki](https://github.com/Kesomannen/gale/wiki/Profile-sync)

### Fixed

- Improved performance of profile imports by only performing the necessary operations
- No warning being shown when a dependency of a disabled mod is uninstalled

## 1.5.12 (2025-05-01)

### Added

- Support for Labyrinthine

### Fixed

- Steam directory name for Lost Skies
- More setup errors related to missing profiles

## 1.5.11 (2025-04-28)

### Added

- Support for Lost Skies and ANEURISM IV

### Fixed

- Window not being resizable on Linux
- Hopefully fixed most issues with Gale crashing on startup

## 1.5.10 (2025-04-23)

### Fixed

- "Invalid signature" error when updating

## 1.5.9 (2025-04-22)

### Added

- Create desktop shortcut option to profile menu
- Support for 8 games:
  - Pulsar: Lost Colony
  - Songs of Conquest
  - White Knuckle
  - Human Fall Flat
  - Magicite
  - ENA: Dream BBQ
  - ASKA

### Fixed

- Punctuation being stripped from config option names
- Gale being registered as a file handler for all file types
- Passing `--no-gui` to a running instance causing the app to crash
- Profile dropdown not being sorted by name

## 1.5.8 (2025-04-15)

### Fixed

- Mod fetching causing a crash in some communities

## 1.5.7 (2025-04-15)

### Added

- Added mod icon to details menu
- Color option editor for config hex codes
- Game information to profile exports, which lets Gale automatically select the correct game when importing profiles

### Fixed

- Minor UI tweaks
- Update R.E.P.O. logo
- Reworked CLI system to fix various bugs
- Some packages not being fetched from small communities

## 1.5.6 (2025-04-01)

### Added

- Local mod drag and drop
- Support for Schedule I

### Fixed

- Compatibility with r2modman on subdirs with differing cases

## 1.5.5 (2025-03-28)

### Fixed

- Some config files freezing the app when opened

## 1.5.4 (2025-03-27)

### Added

- New logo!
- Custom theme colors
- Ability to create profiles at any path
- File association for `.r2z` profile files
- Drag and drop functionality for `.r2z` files
- Ability to select a target profile when using the `Install with Mod Manager` button

### Fixed

- Underscores being removed from config file names
  - They are now replaced with spaces

## 1.5.3 (2025-03-24)

### Fixed

- Moving data folder from default location causing errors

## 1.5.2 (2025-03-22)

### Fixed

- Weird behaviour with deleted profiles coming back

## 1.5.1 (2025-03-22)

### Fixed

- "Invalid signature" error when updating
- Mod installation state not updating when switching profiles

## 1.5.0 (2025-03-22)

### Added

- Support for 5 new games:
  - Odd Remedy
  - My Dream Setup
  - Monster Train 2
  - Disco Elysium
  - Zort
- New data store solution based on SQLite, instead of various json files
  - Your data will automatically be migrated after updating
  - The legacy json files will remain on your filesystem, but won't be updated or read by Gale anymore
- Primary color setting
- Automatic steam library detection by reading steam's `libraryfolders.vdf` file
- Donation link to About menu

### Fixed

- Alphabetical and author sort mods separating upper and lowercase letters
- Hopefully fixed linux launch issues related to Wine/Proton/Whiskey

### Removed

- Steam library setting as it's now redundant

## 1.4.3 (2025-03-11)

### Fixed

- The `dotnet` directory not being copied when launching BepInEx IL2CPP games
- Corrected default r2modman import directory for Risk Of Rain 2
- Valheim launching with the .sh file instead of the .exe on Windows
- MelonLoader rebuilding on each launch
- Small UI changes

## 1.4.2 (2025-03-02)

### Added

- Support for Gang Beasts

### Fixed

- Profile duplication causing the profiles to be linked

## 1.4.1 (2025-03-01)

### Fixed

- Incorrect steam directory name for R.E.P.O.

## 1.4.0 (2025-02-28)

### Added

- Support for I Am Your Beast, MiSide and R.E.P.O.
- Confirmation dialog before resetting config entries

### Fixed

- Typo in telemetry setting name (again)
- Profile duplication causing more disk space to be used than necessary
- Inconsistent profile sorting order on some file systems/platforms
- Small gap between the taskbar and window while the app is maximized
- Decreased the number of disk writes when reordering mods
- Various changes to make the interface more responsive and easier to understand

## 1.3.1 (2025-01-28)

### Fixed

- Typo in telemetry setting name
- Modpack icon selector not working

## 1.3.0 (2025-01-19)

### Added

- Ability to change the path to import r2modman/Thunderstore Mod Manager profiles from
- Mod install time estimate
- Support for 10 new games:
  - Distance
  - Five Nights at Freddy's: Into the Pit
  - GoreBox
  - Hard Time III
  - Old Market Simulator
  - Paquerette Down the Bunborrows
  - Shapez 2
  - Subterranauts
  - Sulfur
  - Tank Team
- Telementry data collection on app startup
  - By updating to this version, you agree to the [Privacy Policy](https://github.com/Kesomannen/gale/blob/master/privacy_policy.md).

### Fixed

- Missing icon for Lycans

## 1.2.2 (2025-01-02)

### Fixed

- Files in untracked subdirs (such as `BepInEx/config`) not being flattened properly

## 1.2.1 (2025-01-01 🎉)

### Added

- Support for 5 games, including the Lovely mod loader:
  - Balatro
  - Hades II
  - Peaks of Yore
  - Subterror
  - STRAFTAT

## Fixed

- ReturnOfModding mods being installed incorrectly
- Launch mode defaulting to `Direct` instead of `Launcher` for first time users
- Unexpected behaviour if an install error occurs while changing the version of a mod

## 1.2.0 (2024-12-04)

### Added

- Support for ATLYSS and Risk Of Rain Returns

## 1.1.5 (2024-11-29)

### Added

- Option to import all files from profiles instead of just standard config extensions
- More logging

### Fixed

- CLI mod installation when the `--no-gui` flag is set
- Automatic game directory detection not working until a setting is changed
- Re-added local DLL installs (only BepInEx for now)
- Profile name case issues on Windows

## 1.1.4 (2024-11-19)

## Added

- Config support to WEBFISHING (TackleBox)

### Fixed

- Older versions of dependencies sometimes being installed
- CLI mod install causing the app to freeze

## 1.1.3 (2024-11-16)

### Added

- A few more game icons
- CLI argument `--install [PATH]` (`-i` for short) which installs a local mod on startup

### Fixed

- Mod updates going through even if the old version failed to uninstall
- Dependency install order, which should fix issues with modpack config being overriden
- Various issues with local mod icons
- Opening game log while the game is running causing an error

## 1.1.2 (2024-11-14)

### Fixed

- Local mod uninstall and toggle throwing errors
- `File is used by another process` error when changing data directory
- Gale profile codes/files causing an error when imported in r2modman

## 1.1.1 (2024-11-14)

### Fixed

- Startup crash due to extra hyphens in mod names (`failed to read profile manifest ...`)
- BepInEx config files installing with extra directories, causing them to seem duplicated
- Uninstalling mods on H3VR throwing a `Failed to delete state file` error
- Re-added Dyson Sphere Program icon

## 1.1.0 (2024-11-13)

### Added

- Experimental support for:
  - 4 platforms: Epic Games, Xbox Store, Oculus and Origin
  - 4 mod loaders: MelonLoader, Shimloader, GDWeave and Northstar
  - 11 games using those mod loaders
- More logging and error context, especially during the startup process
- Tons of game icons
- Links to the new Discord server

### Fixed

- Modpack export causing the app to freeze on games without a "Modpack" category
- Arcus Chroma and Subnautica Below Zero not working at all

## 1.0.1 (2024-11-09)

### Fixed

- Importing profiles with local mods (they are ignored now)
- r2modman/Thunderstore Mod Manager profile transfer for Risk Of Rain 2

## 1.0.0 (2024-11-09)

### Added

- `Open game directory` button to File menu

## 0.9.2 (2024-10-29)

### Added

- Accent color setting

### Changes

- Minor UI tweaks

### Fixed

- Zoom in hotkey (`Ctrl +`)
- Empty string config options causing an extra `=` to be added

## 0.9.1 (2024-10-24)

### Added

- Support for TCG Card Shop Simulator

### Removed

- Redudant options in the Window menu

### Fixed

- Re-added the ability to edit mod versions
- Being able to open multiple mod context menus at the same time
- Importing local mods with BOM-encoded manifests
- IL2CPP BepInEx packs being extracted incorrectly
- Small UI fixes

## 0.9.0 (2024-10-21)

### Added

- Right-click mod context menu (same as the dropdown on the details panel)

### Changed

- Various UI changes and fixes, including revamped reordering

### Fixed

- Symlinks not having .old appended to them when disabling mods
- BOM-encoded config files causing a parse error
- Extra whitespace around config options causing a parse error
- Install with mod manager button on thunderstore.io
- Profile mod count not updating when a mod is uninstalled
- Extra decimals sometimes being added when dragging a config slider

## 0.8.11 (2024-10-14)

### Added

- More game icons

### Changed

- Various UI changes

### Fixed

- Direct launch mode
- Profile mod count not updating when a mod is installing

## 0.8.10 (2024-10-11)

### Changed

- Moved the `Check for updates` button to a new `About Gale` dialog
- Also look for .sh, .x86_64 and .x86 executables when launching games, instead of only .exe files

### Fixed

- Modpack icon selector throwing an error after selecting a file
- Cancelling a mod disable dialog causing the target mod to behave as if it was disabled

## 0.8.9 (2024-10-04)

### Changed

- Minor UI improvements

### Fixed

- Various issues with importing and managing local mods

## 0.8.8 (2024-10-02)

### Fixed

- Uninstalling and toggling mods not affecting the files
- Open directory mod context menu option

## 0.8.7 (2024-10-02)

### Fixed

- Mod list throwing errors before the `Enabled` filter is interacted with

## 0.8.6 (2024-10-02)

### Added

- Option to filter out enabled mods
- Keyboard shortcuts for some of the menu items
- Custom launch arguments
- Window menu

### Removed

- Home page with changelog. The default when starting the app is now the profile page instead.

### Changed

- UI tweaks and fixes

### Fixed

- Last updated showing NaN on Linux
- Sometimes being able to scroll the whole window
- Modpack updates not overriding existing config files

## 0.8.5 (2024-09-11)

### Added

- "Uninstall disabled mods" button to Profile menu
- Code highlighting to markdown code blocks

### Changed

- New monospace font
- Enable gzip for API requests, which can significantly speed up mod fetching
- Mods are now always fetched on startup, even if the automatic fetch setting is disabled
- Minor UI changes

### Fixed

- Local mods not showing in the mod list until the page is refreshed
- The "Install date" sorting mode being affected by mod updates

## 0.8.4 (2024-09-03)

### Fixed

- Opening a config file freezing the app

## 0.8.3 (2024-09-03)

### Added

- File size to mod details menu

### Fixed

- Install with mod manager button on Thunderstore not working
- Mod list installed indicator being hidden
- Delete button not working for non-cfg config files
- Improve error messages for failed modpack uploads
- Zip archives with backslashes instead of frontslashes extracting incorrectly on Unix platforms

## 0.8.2 (2024-08-30)

### Changed

- Install the root mod before dependencies, which should fix issues with modpack config files sometimes being skipped

## 0.8.1 (2024-08-30)

### Removed

- Settings toggle to disable mod download cache, as it now doesn't provide any benefit

### Fixed

- Old mod cache files from pre-0.8 not being deleted on startup. **Note: if you've already updated to 0.8.0, you need to manually delete them.**
- Modpack arguments not being saved when app is closed or profile is changed while the page is open
- `<details>` elements in markdown rendering incorrectly

## 0.8.0 (2024-08-30)

### Added

- Profile > Copy launch arguments
- Indicator for installed mods in the mod list (thanks @DaXcess)

### Removed

- Cache directory setting - mods are now cached within the data directory instead

### Changed

- Use hard links instead of copying files when installing mods, which reduces disk usage and install times significantly
- New cache format for mods (existing cache will be deleted on first launch)
- Use a broad-first search for dependencies instead of depth-first

### Fixed

- Multiple .old extensions sometimes being added when a mod is disabled
- Missing or corrupted profile manifests causing a crash on startup

## 0.7.7 (2024-08-25)

### Added

- Profile > Copy debug info button for easier troubleshooting

### Changed

- Hide orphaned config entries in the config editor
- List other common config file formats in the config editor

### Fixed

- Re-added CLI (oops)
- FINALLY fixed the "Install with Mod Manager" button on Thunderstore
- Various issues related to config file names with non-UTF8 characters
- Multiple config files from the same mod being displayed with the same name
- Decrease config load time by reading files in parallel
- Various UI issues

## 0.7.6 (2024-08-22)

### Removed

- Temp directory setting in favor of using the system's temp directory with automatic cleanup

### Fixed

- Parity with r2modman's mod structure format, which should solve gale-specific mod issues
- Failed r2modman profile imports creating corrupted profiles
- Crash when changing the data directory setting
- Improve profile and export modpack performance by doing more work in-memory

## 0.7.5 (2024-08-20)

### Changed

- Move deleted config files to the trash instead of permanently deleting them

### Fixed

- BepInEx being installed incorrectly on certain games
- Sorting and filtering options not persisting between sessions
- Expand config entry button covering the end of the text input
- Hopefully fixed issues related to moving the data, temp or cache directories

## 0.7.4 (2024-08-09)

### Fixed

- Custom sort order not responding to Descending/Ascending
- Mod reordering moving mods in the wrong direction
- Expanded config entries not being editable in text mode

## 0.7.3 (2024-08-08)

### Added

- Option to disable automatic thunderstore fetching
- Option to ignore updates for the "Update all" banner

### Changed

- Detect steam installation path on Windows from registry
- Sort pinned mods to the top of the mod list
- UI tweaks

### Fixed

- Open gale log button not working
- Thunderstore API token not persisting between sessions
- Modpack export arguments sometimes not being saved
- App closing immediately when a startup error occurs instead of showing an error dialog

## 0.7.2 (2024-08-01)

### Changed

- Switched UI font to a non-rounded version

### Fixed

- Crash when entering the config editor
- Path settings not saving correctly
- Decrease latency when moving data, temp or cache directories on the same volume
- Auto updater not showing up (_not retroactive; you still need to manually update to this version_)
- Additional files like mod manifests being included in file, code and modpack exports

## 0.7.1 (2024-07-30)

### Fixed

- "Invalid type: map ..." errors when importing profile from file and editing path settings

## 0.7.0 (2024-07-30)

### Added

- Simple CLI for choosing game, profile and launching games through the command line
- Resizable text input and list editor for long config entries
- Changelog field and automatic changelog generation in modpack exporter
- More game icons ([#58](https://github.com/Kesomannen/gale/pull/58) and [#61](https://github.com/Kesomannen/gale/pull/61))
- Local mod import from zip ([#57](https://github.com/Kesomannen/gale/pull/57))

### Changed

- Various UI tweaks

### Fixed

- Restricted profile names further, which should fix "failed to read profile manifest" error on startup
- "BepInEx preloader not found" error when launching IL2CPP games
- Window size and position not persisting between sessions
- Having many profiles causing the profile dropdown to go off-screen
- Crash when changing the version of certain mods
- Dependencies not being installed when changing the version of a mod
- Local mod icons not being displayed (finally)
- "Failed to execute 'query_mods_in_profile'" error when viewing a profile with deleted mods
- "Visiting" a new game and immediately switching back causing a "Failed to read game save data" on the next launch

## 0.6.2 (2024-07-15)

### Added

- Ctrl+click shortcut to install a mod

### Changed

- Reworked config entry sliders, which should fix all issues with freezing or crashing
- Search mods by their description as well as name

### Fixed

- Welcome dialog opening every time the app is launched until a setting is changed
- Disabling mods with missing dependencies
- Deleted config files not being removed from the modpack export list
- Moving temp, data or cache directory to a different drive
- Mods being imported in the reverse order when importing a Gale-generated profile code

## 0.6.1 (2024-07-03)

### Added

- Preview of contents before importing a profile

### Changed

- Display a list of mod cards instead of table when viewing Dependants/Dependencies

### Fixed

- BepInEx/\* directories in remote mods being installed at the wrong location
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
- BepInEx/\* directories in local mods being installed at the wrong location

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

- Auto updater not working (_not retroactive; you still need to manually update to this version_)
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
