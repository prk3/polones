# makes a static build of SDL
# expects:
#   - /sdl-source to contain SDL source code
#   - /sdl-config to be an empty dir for SDL build config files
#   - /sdl-build to be an empty dir for build artifacts
# based on https://github.com/libsdl-org/SDL/blob/main/docs/README-linux.md
FROM ubuntu:22.04
COPY build-sdl-linux-amd64.sh /
RUN apt update && \
    apt install -y build-essential make \
        pkg-config cmake ninja-build gnome-desktop-testing libasound2-dev libpulse-dev \
        libaudio-dev libjack-dev libsndio-dev libx11-dev libxext-dev \
        libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev \
        libxkbcommon-dev libdrm-dev libgbm-dev libgl1-mesa-dev libgles2-mesa-dev \
        libegl1-mesa-dev libdbus-1-dev libibus-1.0-dev libudev-dev fcitx-libs-dev \
        libpipewire-0.3-dev libwayland-dev libdecor-0-dev liburing-dev
ARG USER=1000 GROUP=1000
USER $USER:$GROUP
CMD ["sh", "build-sdl-linux-amd64.sh"]
