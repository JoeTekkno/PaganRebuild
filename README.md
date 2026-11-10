# PaganRebuild

A newly written engine for **Ultima VIII: Pagan**, in Rust. It plays the game straight from the original data, without DOSBox, and adds improvements you can switch on and off.

Project page: https://scareglow23.github.io/PaganRebuild/

**No game data is included.** You need your own copy of the game, for example the [Ultima 8 Gold Edition on GOG](https://www.gog.com/en/game/ultima_8_gold_edition).

## Installation

1. Download the zip from [Releases](https://github.com/scareglow23/PaganRebuild/releases).
2. Extract it into the Ultima 8 folder (where `ENGLISH\` is).
3. Run `PaganRebuild\PaganRebuild.exe` (German version: `--lang german`, French: `--lang french`).

Windows 64-bit. Linux and macOS are not tested yet. Savegames and settings are kept in `%APPDATA%\pagan`.

## Album music (optional)

PaganRebuild can play the album "Ultima VIII: Pagan - Orchestral Re-Arrangement" instead of the MIDI music. The album is not included:

1. Download the album from [Bandcamp](https://dustinnaegel.bandcamp.com/album/ultima-viii-pagan-orchestral-re-arrangement) as "Ogg Vorbis".
2. Extract the zip into a folder `album` inside the Ultima 8 folder (next to `ENGLISH\`).
3. In the options menu under Sound, set "Music" to "Album".

Original music: Nenad Vugrinec<br>
Arrangement and orchestration: Dustin Naegel<br>
https://dustinnaegel.bandcamp.com/album/ultima-viii-pagan-orchestral-re-arrangement

## Building

Rust (stable) is needed. On Windows the GNU toolchain is used (`stable-x86_64-pc-windows-gnu`).

```
cargo build --release
```

The program is `target/release/PaganRebuild.exe`. It finds the game data by looking in the folders above it; otherwise give the folder with `--game <dir>`. All options: `PaganRebuild --help`.

## Credits

Based on the work of ScummVM (formerly Pentagram), https://www.scummvm.org/.<br>
SoundFont: GeneralUser GS by S. Christian Collins, https://www.schristiancollins.com/generaluser.php.

Ultima VIII: Pagan (c) 1994 Origin Systems / Electronic Arts. Fan project, not affiliated with Electronic Arts or Origin Systems.

PaganRebuild is developed with the help of Claude Code (Anthropic).

## License

GPL-3.0-or-later, see [LICENSE](LICENSE).
