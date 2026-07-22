# GLAP

GameLord's Audio Player.

Let's be honest. Audio player apps are not fun to use. One day I thought, why not just make my own at this point. This project was originally RMIDI, a dedicated MIDI player, but that didn't work out for reasons I won't get into (I hate MIDI status `0xB0` now). ~~It's written in rust so you know it's fast.~~

Oh also main_old.rs contains the old code if you want to poke it.

## What did I test it on

~~At the time of writing (2026-07-17 at 1:34PM EST) I haven't tested it, but plan to test it on my personal music library (mostly soundtracks to games convienently comprised of Mpeg Layer 3 and Vorbis files (I'm to lazy to download the FLACs from the steam soundtracks atm)).~~

Known to work on:

- Mpeg-1 Audio Layer II
- Mpeg-1 Audio Layer III
- Free Lossless Audio Codec
- Ogg Vorbis

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
