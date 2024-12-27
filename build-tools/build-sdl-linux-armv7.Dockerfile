# makes a static build of SDL
# expects:
#   - /sdl-source to contain SDL source code
#   - /sdl-config to be an empty dir for SDL build config files
#   - /sdl-build to be an empty dir for build artifacts

# we compile on debian oldstable to depend on a pretty old glibc version.
FROM debian:bullseye

# some libs (e.g. libdecor-0-dev) are only available in backports repo.
RUN printf "\ndeb http://ftp.debian.org/debian bullseye-backports main\n" >> /etc/apt/sources.list

# the list of packages comes from SDL2 docs:
# https://github.com/libsdl-org/SDL/blob/SDL2/docs/README-linux.md
RUN apt update && apt install -y build-essential git make autoconf automake libtool \
    pkg-config cmake ninja-build gnome-desktop-testing libasound2-dev libpulse-dev \
    libaudio-dev libjack-dev libsndio-dev libsamplerate0-dev libx11-dev libxext-dev \
    libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev libwayland-dev \
    libxkbcommon-dev libdrm-dev libgbm-dev libgl1-mesa-dev libgles2-mesa-dev \
    libegl1-mesa-dev libdbus-1-dev libibus-1.0-dev libudev-dev fcitx-libs-dev \
    libpipewire-0.3-dev libdecor-0-dev \
    # curl is needed for installing rustup.
    curl

ARG USER=1000 GROUP=1000
USER $USER:$GROUP
CMD ["sh", "-c", "cd /sdl-config && /sdl-source/configure --prefix=/sdl-build --with-pic && make -j4 && make install"]
