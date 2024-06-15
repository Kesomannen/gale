import os
from os import path
from PIL import Image

SIZE = (256, 256)

if __name__ == '__main__':
    dir = path.join('static', 'games')
    for file in os.listdir(dir):
        with Image.open(path.join(dir, file)) as img:
            img.thumbnail(SIZE)

            name, ext = path.splitext(file)
            img.save(path.join(dir, name + '.webp'), optimize=True)