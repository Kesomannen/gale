# Resizes and converts files in the static/games directory to 256x256 webp

import os
from os import path
from PIL import Image

SIZE = (256, 256)

if __name__ == '__main__':
    dir = path.join(__file__, '..', '..', 'static', 'games')
    dir = path.realpath(dir)

    for file in os.listdir(dir):
        absolute = path.join(dir, file)

        with Image.open(absolute) as img:
            img.thumbnail(SIZE)

        os.remove(absolute)
        name, ext = path.splitext(file)
        img.save(path.join(dir, name + '.webp'), optimize=True)