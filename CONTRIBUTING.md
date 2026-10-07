# Contributing to Gale

### Issues

#### Reporting a bug

Before submitting a bug, search the list of issues for users who have already reported similar problems. Make sure to include closed issues are included in your search. If it isn't reported yet, open an issue and include the following information, if relevant:

- Operating system
- Installation method
- Gale log file, available from the menubar option `File > Open Gale log`
  - If you can't use the interface, you can instead find the file at `%appdata%/com.kesomannen.gale/latest.log` (Windows) and `~/.local/share/com.kesomannen.gale/latest.log` (Linux).
- Game log file, available from the menubar option `File > Open game log`

#### Requesting a feature

As with bugs, first make sure to search the existing issues for duplicates before submitting a new feature request. If an issue already exists for your issue, feel free to support it with a reaction or leave a comment.

### Pull requests

Code contributions are welcome. However, if you are aiming to implement a larger or more disruptive feature, please get in contact with me (Kesomannen) on Discord first.

You may use AI to assist you, but you are always responsible for your work and must write the description and any follow-up messages yourself. Fully AI generated or autonomous pull requests are not permitted.

Game additions should be done using the `add_game.py` script. You can find instructions on [the wiki](https://github.com/Kesomannen/gale/wiki/Adding-games).

Adding a locale is mostly straightforward, just make sure to add the language code to the `settings.json` file. For reference, see [the Azerbaijani PR](https://github.com/Kesomannen/gale/pull/715).
