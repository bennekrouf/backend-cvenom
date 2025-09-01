#!/bin/bash
curl -L -o fontawesome-free.zip https://use.fontawesome.com/releases/v6.6.0/fontawesome-free-6.6.0-desktop.zip
unzip fontawesome-free.zip -d fontawesome
cp fontawesome/fontawesome-free-6.6.0-desktop/otfs/Font\ Awesome\ 6\ Brands-Regular-400.otf ~/Library/Fonts/
atsutil databases -remove
rm -rf fontawesome fontawesome-free.zip
