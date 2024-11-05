# Reorders the games.json file to the order shown in Gale's UI 
# (alphabetical with popular games at the top)

from os import path
import json
from re import sub

if __name__ == '__main__':
    json_path = path.join(__file__, '..', '..', 'src-tauri', 'games.json')
    json_path = path.realpath(json_path)

    with open(json_path, 'r') as file:
        text = file.read()
    
    games = json.loads(text)
    sorted = sorted(games, key=lambda game: (not (game.get('popular') or False), game['name']))

    with open(json_path, 'w') as file:
        file.write(json.dumps(sorted, indent=4))