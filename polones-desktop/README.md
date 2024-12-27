# Polones desktop

This app lets you play NES games on your computer. It's rather bare-bones, expect more features in the future.

## Installation

Download a binary for your platform from the [releases](https://github.com/prk3/polones/releases) page. On Linux you have to drop it in a directory with executables, e.g. `/usr/local/bin`, and make it executable (`chmod +x ...`).

Here are available options:

| file                  | Platform                                                 |
|-----------------------|----------------------------------------------------------|
| polones-desktop-amd64 | 64 bit Linux desktops                                    |
| polones-desktop-arm64 | 64 bit Linux on Raspberry Pi (3, 4, 400, 5, 500, Zero 2) |
| polones-desktop-armv7 | 32 bit Linux on Raspberry Pi (as above + 2)              |

If you're on a different platform, you'll have to build polones-desktop from source.

## Running a game

Execute the program, passing path to a game file as the first argument.

```sh
polones-desktop /path/to/a/game.nes
```

## Controls

Gamepad 1

| NES pad button | Input key |
|----------------|-----------|
| Up             | W         |
| Down           | S         |
| Left           | A         |
| Right          | D         |
| Select         | R         |
| Start          | T         |
| B              | F         |
| A              | G         |

Gamepad 2

Not connected
