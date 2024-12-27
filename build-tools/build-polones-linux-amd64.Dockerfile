# builds polones-dekstop for linux amd64
# expects:
#   - /polones to contain polones source code
#   - /sdl-build to contain SDL build artifacts

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

# rustup is installed in user's home directory, so we must set up a user with home.
ARG USER=1000 GROUP=1000
RUN groupadd --gid $GROUP rust && useradd --uid $USER --gid $GROUP --create-home rust
USER $USER:$GROUP

# we now install rustup and the desired rust toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.83.0 -y

WORKDIR /polones/polones-desktop
ENV PKG_CONFIG_PATH=/sdl-build/lib/pkgconfig/
ENV RUSTFLAGS="-C relocation-model=pic"

# source of rustup env and build must happen in one command.
CMD ["sh", "-c", ". \"$HOME/.cargo/env\" && cargo build --target x86_64-unknown-linux-gnu --features sdl2/static-link,sdl2/use-pkgconfig --profile release"]
