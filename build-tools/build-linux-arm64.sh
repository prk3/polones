#!/bin/sh
set -e
rm -rf build
mkdir build build/sdl-config-arm64 build/sdl-build-arm64
git clone https://github.com/libsdl-org/SDL.git --branch release-2.30.10 --depth 1 build/sdl-source
docker build --platform linux/arm64 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-sdl-linux-arm64.Dockerfile -t build-sdl-linux-arm64:latest ./build-tools
docker run --platform linux/arm64 -v ./build/sdl-source:/sdl-source -v ./build/sdl-config-arm64:/sdl-config -v ./build/sdl-build-arm64:/sdl-build build-sdl-linux-arm64:latest
docker build --platform linux/arm64 --build-arg USER=$(id -u) --build-arg GROUP=$(id -g) -f ./build-tools/build-polones-linux-arm64.Dockerfile -t build-polones-linux-arm64:latest ./build-tools
docker run --platform linux/arm64 -v ./build/sdl-build-arm64:/sdl-build -v ./:/polones build-polones-linux-arm64:latest
