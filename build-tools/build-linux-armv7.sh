#!/bin/sh
set -e
rm -rf build
mkdir build build/sdl-config-armv7 build/sdl-build-armv7
git clone https://github.com/libsdl-org/SDL.git --branch release-2.30.10 --depth 1 build/sdl-source
docker build --platform linux/arm/v7 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-sdl-linux-armv7.Dockerfile -t build-sdl-linux-armv7:latest ./build-tools
docker run --platform linux/arm/v7 -v ./build/sdl-source:/sdl-source -v ./build/sdl-config-armv7:/sdl-config -v ./build/sdl-build-armv7:/sdl-build build-sdl-linux-armv7:latest
docker build --platform linux/arm/v7 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-polones-linux-armv7.Dockerfile -t build-polones-linux-armv7:latest ./build-tools
docker run --platform linux/arm/v7 -v ./build/sdl-build-armv7:/sdl-build -v ./:/polones build-polones-linux-armv7:latest