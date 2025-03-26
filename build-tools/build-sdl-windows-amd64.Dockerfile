# makes a static build of SDL
# expects:
#   - /sdl-source to contain SDL source code
#   - /sdl-config to be an empty dir for SDL build config files
#   - /sdl-build to be an empty dir for build artifacts

# we compile on debian oldstable, as we do for other platforms
FROM debian:bullseye

RUN apt update && apt install -y \
    build-essential git make autoconf automake libtool pkg-config cmake ninja-build \
    gcc-mingw-w64-x86-64 \
    # curl is needed for installing rustup.
    curl

ARG USER=1000 GROUP=1000
USER $USER:$GROUP
CMD ["sh", "-c", "cd /sdl-config && /sdl-source/configure --host=x86_64-w64-mingw32 --prefix=/sdl-build && make -j4 && make install"]
