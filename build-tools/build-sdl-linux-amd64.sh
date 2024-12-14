#!/bin/sh
cd /sdl-config
/sdl-source/configure --prefix=/sdl-build --with-pic
make -j4
make install
