from os import path
import os
import requests

from utils import kebab_case, read_games, write_games
from transform_icons import transform_icons
from sort_games import sort_games

def check_slug(slug):
    print(f'Checking slug {slug} on thunderstore.io')
    response = requests.get(f'https://thunderstore.io/c/{slug}/')
    return response.status_code != 404

def multiple_choice(prompt, options):
    while True:
        print(prompt)
        for i, option in enumerate(options):
            print(f'{i + 1}: {option}')
        choice = input("> ")
        
        try:
            return options[int(choice) - 1]
        except:
            print('Invalid input, try again')

if __name__ == '__main__':
    name = input("Enter the game's name:\n> ")
    slug = kebab_case(name)
    if not check_slug(slug):
        while True:
            slug = input("Enter the game's slug on thunderstore.io:\n> ")
            
            if check_slug(slug):
                break
            else:
                print("Slug not found on thunderstore, try again")
        
    mod_loader = multiple_choice(f'Which mod loader does {name} use?', ['BepInEx', 'MelonLoader', 'GDWeave', 'Shimloader', 'Lovely', 'Northstar', 'ReturnOfModding'])
    
    platforms = {}
    all_platforms = ['steam', 'epicGames', 'xboxStore', 'oculus', 'origin', 'done']
    
    while len(all_platforms) > 1:
        new_platform = multiple_choice(f'Which platform is {name} available on?', all_platforms)
        if new_platform == 'done':
            break
        elif new_platform == 'steam':
            id = int(input("What's the game's internal steam ID?\n> "))
            platforms['steam'] = { 'id': id }
            
            dirName = input(f"What's {name}'s internal folder name on steam (leave blank to use \"{name}\")?\n> ")
            
            if len(dirName) > 0:
                platforms['steam']['dirName'] = dirName
        elif new_platform == 'epicGames' or new_platform == 'xboxStore':
            id = input(f"What's {name}'s internal identifier on {new_platform}?\n> ")
            platforms[new_platform] = { 'identifier': id }
        else:
            platforms[new_platform] = {}
            
        all_platforms.remove(new_platform)
    
    print(f'Add a square image to static/games with the name "{slug}.png/jpg/jpeg/webp"')
    while True:
        input('(press enter to continue)')
        
        found = False
        for ext in ['png', 'jpg', 'jpeg', 'webp']:
            icon_path = path.join(__file__, '..', '..', 'static', 'games', f'{slug}.{ext}')
            icon_path = path.realpath(icon_path)
            print(f'Checking for icon at {icon_path}')
            if path.isfile(icon_path):
                found = True
                break
            
        if found:
            print('Icon found, resizing and optimizing...')
            break
        
        print('Icon not found, try again')
    
    transform_icons()
    
    game = {
        'name': name,
        'slug': slug,
        'modLoader': {
            'name': mod_loader
        },
        'platforms': platforms
    }
    
    print(f'Inserting into games.json...')
    
    (json_path, games) = read_games()
    games.append(game)
    write_games(json_path, games)
    
    sort_games()
    
    print(f'Inserted {name} into games.json')