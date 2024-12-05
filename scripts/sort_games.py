# Reorders the games.json file to the order shown in Gale's UI 
# (alphabetical with popular games at the top)

from os import path
import json

from utils import read_games, write_games

def sort_games():
    (json_path, games) = read_games()
    
    sort = sorted(games, key=lambda game: (not (game.get('popular') or False), game['name']))

    write_games(json_path, sort)

if __name__ == '__main__':
    sort_games()