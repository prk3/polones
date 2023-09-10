![logo-nsf-polonez](./assets/polones-logo-830.png)

# Polones

Polones is a NES (Nintendo Entertainment System) emulator written in Rust. It currently offers applications for two platforms: Linux desktop and web. The core functionality of the emulator is available in `polones-core` library, and can be used to easily port Polones to any platform.

# Compatibility

The following are some of many games Polones can emulate:

- Super Mario Bros.
- Super Mario Bros. 2
- Super Mario Bros. 3
- The Legend of Zelda
- Prince of Persia
- Contra
- Donkey Kong
- Tetris
- Ice Climber
- Battle City
- Galaxian
- Arkanoid
- Excitebike
- Pac-Man

# Modules

This repository contains the following modules:

| Name              | Description                                                                                 |
|-------------------|---------------------------------------------------------------------------------------------|
| polones-core      | Library for NES console emulation.                                                          |
| polones-desktop   | NES emulator app for Linux desktop built with SDL2 library. Used for debugging the library. |
| polones-web       | NES emulator app for the web. Available online at <https://prk3.github.io/polones/>.        |
| polones-assembler | NES program assembler. Used for writing roms testing the emulator library.                  |
| polones-test      | Tool for testing and benchmarking the emulator library.                                     |

# TODO

- Implement APU DMC channel to play DCM sounds
- Find better audio resampling algorithm to improve audio quality
- Implement more mappers to support more games
- Make applications more user-friendly
- Add convenience functionality (alternative color palettes, dumping emulator state, persisting battery-backed game memory)

# License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
