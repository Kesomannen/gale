# Reads the game.json file and generates an array with all the games' slugs
# Useful for thunderstore.toml

from os import path
import json

from utils import kebab_case

if __name__ == '__main__':
    json_path = path.join(__file__, '..', '..', 'src-tauri', 'games.json')
    json_path = path.realpath(json_path)

    with open(json_path, 'r') as file:
        text = file.read()

    print('[')
    
    games = json.loads(text)
    for game in games:
        try:
            slug = game['slug']
        except KeyError:
            slug = kebab_case(game['name'])

        print(f'    "{slug}",')

    print(']')