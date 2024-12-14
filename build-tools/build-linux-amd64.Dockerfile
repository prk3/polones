# builds polones-dekstop for linux amd64
# expects:
#   - /polones to contain polones source code
#   - /sdl-build to contain SDL build artifacts
FROM rust:1.83.0-bullseye
WORKDIR /polones/polones-desktop
ARG USER=1000 GROUP=1000
USER $USER:$GROUP
ENV PKG_CONFIG_PATH=/sdl-build/lib/pkgconfig/
ENV RUSTFLAGS="-C relocation-model=pic"
CMD ["cargo", "build", "--release", "--features=sdl2/static-link,sdl2/use-pkgconfig"]

