# GLAP

GameLord's Audio Player.

Let's be honest. Audio player apps are not fun to use. One day I thought, why not just make my own at this point. This project was originally RMIDI, a dedicated MIDI player, but that didn't work out for reasons I won't get into (I hate MIDI status `0xB0` now). ~~It's written in rust so you know it's fast.~~

Oh also `src/main_old.rs` contains the old code if you want to poke it.

## Requirements and notes

You need a [nerd font](https://www.nerdfonts.com/#home) for the controls to render properly, some known side-effects of not having one are:

- Highlighting doesn't work on Windows Terminal or Conhost if you aren't using a nerd font.

It is known to work on Windows Terminal and Conhost. In terminal.app hex code color schemes may not work at all. (see [this](https://github.com/ratatui/ratatui/issues/475) issue for more information).

Additionally, there are some cases of extreme ram usage (>50% in some cases where I measured it) with an IGPU when using Windows Terminal due to how it allocates ram, __this isn't my fault, it is the fault of the terminal app__. Turning on WARP (the software renderer) usually mitigates this.

Additionally, __THIS IS NOT A MUSIC STREAMING APP__.

## Configuration options

- `music_folder` overrides the default music folder just in case you aren't using your OS'es default music folder (e.g. `C:\Users\USER\OneDrive\Music` for some reason or something idk)
- `dont_filter_unplayable` when set to true doens't filter out unplayable music, probably to test an extension that I don't know if it works.
- `theme` a table consisting of:
  - `foreground` a hex code, color name, or ANSI color index setting the foreground (text) color
  - `background` the same thing but it sets the background color
  - `controls` the same thing but it sets the controls color
  - `playbar` the same thing but it sets the playbar played thing color

## What did I test it on

~~At the time of writing (2026-07-17 at 1:34PM EST) I haven't tested it, but plan to test it on my personal music library (mostly soundtracks to games convienently comprised of Mpeg Layer 3 and Vorbis files (I'm to lazy to download the FLACs from the steam soundtracks atm)).~~

Known to work on:

- Mpeg-1 Audio Layer II (`.mp2`)
- Mpeg-1 Audio Layer III (`.mp3`)
- Free Lossless Audio Codec (`.flac`)
- Ogg Vorbis (`.ogg`)
- AAC-LC (`.aac`)

Should theoretically work on anything that [Symphonia](https://github.com/pdeljanov/Symphonia) supports, though some formats (namely Mpeg-1 Audio Layer I, raw PCM, and others) are hard for me to get my hands on so there is no guarentee that anything that I haven't tested works.

## Note to any of the shipwrights reading this (iykyk)

Rupnil said I could rebrand this I have specific permission from him I can dm you a link to the thread if you want me to.

## Building yourself

- Follow the instructions on [this](https://github.com/Rust-SDL2/rust-sdl2) page for your platform
- Run:

```Bash
cargo build
```

- That's it.
