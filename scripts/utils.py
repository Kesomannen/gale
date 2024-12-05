import json
from os import path
from re import sub

def kebab_case(s):
  return '-'.join(
    sub(r"(\s|_|-)+"," ",
    sub(r"[A-Z]{2,}(?=[A-Z][a-z]+[0-9]*|\b)|[A-Z]?[a-z]+[0-9]*|[A-Z]|[0-9]+",
    lambda mo: ' ' + mo.group(0).lower(), s)).split())
  
def read_games():
    json_path = path.join(__file__, '..', '..', 'src-tauri', 'games.json')
    json_path = path.realpath(json_path)

    with open(json_path, 'r') as file:
        text = file.read()
    
    games = json.loads(text)
    
    return (json_path, games)
    
def write_games(json_path, games):
    with open(json_path, 'w') as file:
        file.write(json.dumps(games, indent=4))