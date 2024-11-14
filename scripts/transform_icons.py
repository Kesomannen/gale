# Resizes and converts files in the static/games directory to 256x256 webp

import os
from os import path
from PIL import Image

SIZE = (256, 256)

if __name__ == '__main__':
    dir = path.join(__file__, '..', '..', 'static', 'games')
    dir = path.realpath(dir)

    for file in os.listdir(dir):
        if file.endswith('webp'):
            continue

        absolute = path.join(dir, file)

        with Image.open(absolute) as img:
            img.thumbnail(SIZE)

            name, ext = path.splitext(file)
            print('saving', name + '.webp')
            img.save(path.join(dir, name + '.webp'), format='webp', optimize=True)

        if not file.endswith('webp'):
            print('removing', file)
            os.remove(absolute)