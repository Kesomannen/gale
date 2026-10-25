from os import path
import json
import os
import time

from termcolor import colored, cprint
import requests

from utils import kebab_case, read_games, write_games
from sort_games import sort_games


def bold(str):
    return colored(str, attrs=["bold"])


BACKEND_URLS = {
    "Thunderstore": "https://thunderstore.io/c/{slug}/",
    "Hexium": "https://{slug}.hexium.gg/",
}


def check_slug(slug, backends):
    for backend in backends:
        url = BACKEND_URLS[backend].format(slug=slug)
        cprint(f"Checking slug {slug} on {backend}", "grey")

        try:
            response = requests.get(url, timeout=15)
            response.raise_for_status()
        except requests.exceptions.RequestException as e:
            cprint(f"Could not find {slug} on {backend}: {e}", "red")
            return False

    return True


def multiple_choice(prompt, options, default=None, title_case=False):
    while True:
        print(prompt)
        for i, option in enumerate(options):
            label = option.title() if title_case else option
            if default != None and i == default:
                print(f"{i + 1}: {label} (hit enter to pick)")
            else:
                print(f"{i + 1}: {label}")
        choice = input("> ")

        try:
            if default != None and len(choice) == 0:
                cprint(f"Picked {options[default]}", "grey")
                return options[default]
            return options[int(choice) - 1]
        except:
            cprint("Invalid input, try again", "red")


def get_steam_app_list():
    CACHE_FILE = "steam_app_list.json"
    CACHE_EXPIRATION = 24 * 60 * 60  # 24 hours

    cache_path = path.join(path.dirname(path.abspath(__file__)), CACHE_FILE)

    if path.exists(cache_path):
        file_mod_time = path.getmtime(cache_path)
        if (time.time() - file_mod_time) < CACHE_EXPIRATION:
            cprint("Using cached Steam app list", "grey")
            with open(cache_path, "r") as f:
                return json.load(f)

    api_url = "https://api.steampowered.com/ISteamApps/GetAppList/v2/"

    try:
        response = requests.get(api_url)
        response.raise_for_status()

        data = response.json()

        with open(cache_path, "w") as f:
            json.dump(data, f)

        return data
    except requests.exceptions.RequestException as e:
        cprint(f"Error fetching app list from Steam API: {e}", "red")
        return None


def get_steam_app_id(game_name):
    app_list = get_steam_app_list()

    if app_list == None:
        return None

    app_list = app_list.get("applist", {}).get("apps", [])

    for app in app_list:
        if app["name"].lower() == game_name.lower():
            id = app["appid"]
            cprint(f"Resolved app ID from Steam API: {id}", "grey")
            return id

    cprint(f"Could not find app in Steam app list", "grey")
    return None


def insert_into_thunderstore_toml(slug):
    toml_path = path.join(path.dirname(__file__), "..", "thunderstore.toml")

    with open(toml_path, "r") as file:
        lines = file.readlines()

    array_start = lines.index("communities = [\n")
    new_line = f'    "{slug}",\n'

    for i, line in enumerate(lines[array_start + 1 : -1]):
        if line > new_line:
            lines.insert(i + array_start + 1, new_line)
            break
    else:
        lines.insert(len(lines) - 1, new_line)

    with open(toml_path, "w") as file:
        file.write("".join(lines))


if __name__ == "__main__":
    name = input("Enter the name of the game:\n> ")
    slug = kebab_case(name)

    backend_choice = multiple_choice(
        f"Where is {name} hosted?",
        ["Thunderstore", "Hexium", "Thunderstore and Hexium"],
        default=0,
    )
    backends = (
        ["Thunderstore", "Hexium"]
        if backend_choice == "Thunderstore and Hexium"
        else [backend_choice]
    )

    if not check_slug(slug, backends):
        while True:
            slug = input(f"Enter the game's slug on {' and '.join(backends)}:\n> ")

            if check_slug(slug, backends):
                break
            else:
                cprint("Slug not found, try again", "red")

    mod_loader = multiple_choice(
        f"Which mod loader does {name} use?",
        [
            "BepInEx",
            "MelonLoader",
            "GDWeave",
            "Shimloader",
            "Lovely",
            "Northstar",
            "ReturnOfModding",
        ],
        default=0,
    )

    platforms = {}
    all_platforms = [
        "steam",
        "epicGames",
        "xboxStore",
        "oculus",
        "origin",
        "no, continue",
    ]

    while len(all_platforms) > 1:
        is_first_pick = len(platforms) == 0

        new_platform = multiple_choice(
            f"Which platform is {name} available on?"
            if is_first_pick
            else f"Is {name} available on any other platforms?",
            all_platforms[:-1] if is_first_pick else all_platforms,
            default=0 if is_first_pick else len(all_platforms) - 1,
            title_case=True,
        )
        if new_platform == "no, continue":
            break
        elif new_platform == "steam":
            id = get_steam_app_id(name)

            if id == None:
                id = int(input(f"What is the steam app ID for {name}?\n> "))

            platforms["steam"] = {"id": id}

            dirName = input(
                f'What is the internal folder name on Steam for {name}?\n(leave blank to use "{name}")\n> '
            )

            if len(dirName) > 0:
                platforms["steam"]["dirName"] = dirName
        elif new_platform == "epicGames" or new_platform == "xboxStore":
            id = input(f"What's {name}'s internal identifier on {new_platform}?\n> ")
            platforms[new_platform] = {"identifier": id}
        else:
            platforms[new_platform] = {}

        all_platforms.remove(new_platform)

    game = {
        "name": name,
        "slug": slug,
        "modLoader": {"name": mod_loader},
        "platforms": platforms,
    }
    if backends != ["Thunderstore"]:
        game["backends"] = backends
    if "Hexium" in backends:
        game["iconUrl"] = f"https://hexium.gg/upload/communities/{slug}.png"

    (json_path, games) = read_games()
    games.append(game)
    write_games(json_path, games)

    sort_games()

    print(f"Inserted {name} into {bold('src-tauri/games.json')}")

    if "Thunderstore" in backends:
        insert_into_thunderstore_toml(slug)

        print(f"Inserted {slug} into {bold('thunderstore.toml')}")

    cprint(f"Done!", "green")
