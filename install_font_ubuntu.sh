#!/bin/bash
curl -L -o fontawesome-free.zip https://use.fontawesome.com/releases/v6.6.0/fontawesome-free-6.6.0-desktop.zip
unzip fontawesome-free.zip -d fontawesome
mkdir -p ~/.fonts/fontawesome
cp fontawesome/fontawesome-free-6.6.0-desktop/otfs/Font\ Awesome\ 6\ Brands-Regular-400.otf ~/.fonts/fontawesome/
fc-cache -f -v
rm -rf fontawesome fontawesome-free.zip
