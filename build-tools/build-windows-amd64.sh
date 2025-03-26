#!/bin/sh
set -e
rm -rf build
mkdir build build/sdl-config-windows-amd64 build/sdl-build-windows-amd64
git clone https://github.com/libsdl-org/SDL.git --branch release-2.30.10 --depth 1 build/sdl-source
docker build --platform linux/amd64 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-sdl-windows-amd64.Dockerfile -t build-sdl-windows-amd64:latest ./build-tools
docker run --platform linux/amd64 -v ./build/sdl-source:/sdl-source -v ./build/sdl-config-windows-amd64:/sdl-config -v ./build/sdl-build-windows-amd64:/sdl-build build-sdl-windows-amd64:latest
docker build --platform linux/amd64 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-polones-windows-amd64.Dockerfile -t build-polones-windows-amd64:latest ./build-tools
docker run --platform linux/amd64 -v ./build/sdl-build-windows-amd64:/sdl-build -v ./:/polones build-polones-windows-amd64:latest
