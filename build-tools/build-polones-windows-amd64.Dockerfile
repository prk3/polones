# builds polones-dekstop for windows amd64
# expects:
#   - /polones to contain polones source code
#   - /sdl-build to contain SDL build artifacts

# we compile on debian oldstable, as we do for other platforms
FROM debian:bullseye

RUN apt update && apt install -y \
    build-essential git make autoconf automake libtool pkg-config cmake ninja-build \
    gcc-mingw-w64-x86-64 \
    # curl is needed for installing rustup.
    curl

# rustup is installed in user's home directory, so we must set up a user with home.
ARG USER=1000 GROUP=1000
RUN groupadd --gid $GROUP rust && useradd --uid $USER --gid $GROUP --create-home rust
USER $USER:$GROUP

# we now install rustup and the desired rust toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain 1.83.0 --target x86_64-pc-windows-gnu -y

WORKDIR /polones/polones-desktop
ENV RUSTFLAGS="-L /sdl-build/lib"

# source of rustup env and build must happen in one command.
CMD ["sh", "-c", ". \"$HOME/.cargo/env\" && cargo build --target x86_64-pc-windows-gnu --features sdl2/static-link --profile release"]
