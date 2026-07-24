extern crate sdl2;

mod audio_player;
mod player;

use std::{default::Default, fs::File, io::stdin};

// use ratatui::DefaultTerminal;
use symphonia::core::{
    codecs::audio::AudioDecoderOptions,
    errors::Error,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

use crate::audio_player::AudioPlayer;

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

// enum Page { // TODO: TUI (RataTUI)
//     Home,
//     About,
//     Player,
//     Settings,
// }

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut path = String::new();
    let p = std::env::args().nth(1);
    match p {
        Some(n) => path = n,
        None => {
            println!("Whar is the file:");
            stdin().read_line(&mut path)?;
        }
    }
    path = path.trim_matches(['\r', '\n', '"', '\'']).to_owned();

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
    }

    let mut v = String::new();
    println!("What do you want the volume to be out of 100?");
    stdin().read_line(&mut v)?;
    let d = v.trim_end().parse::<u8>().unwrap_or(100_u8);
    let volume = d as f32 / 100_f32;

    let file = Box::new(File::open(path).unwrap());
    let mss = MediaSourceStream::new(file, Default::default());

    let hint = Hint::new();

    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();
    let dec_opts: AudioDecoderOptions = Default::default();

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .unwrap();

    let track = format.default_track(TrackType::Audio).unwrap().clone();

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(
            track.codec_params.as_ref().unwrap().audio().unwrap(),
            &dec_opts,
        )
        .unwrap();
    let track_id = track.id;
    let mut samples: Vec<f32> = Default::default();

    #[cfg(debug_assertions)]
    {
        println!("Starting audio sample parsing. This can take a fat minute in dev so be patient.");
    }
    while let Some(packet) = format.next_packet().unwrap() {
        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let mut t: Vec<f32> = Default::default();
                t.resize(audio_buf.samples_interleaved(), 0_f32);
                audio_buf.copy_to_slice_interleaved(&mut t);
                samples.append(&mut t);
            }
            Err(Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }
    if volume != 1_f32 {
        samples = samples.iter().map(|f| f * volume).collect::<Vec<f32>>()
    }
    println!("{:?}", samples.len());

    let binding = track.codec_params.unwrap();
    let info = binding.audio().unwrap();
    let sample_rate = info.sample_rate.unwrap();
    let channels_count = info.channels.to_owned().unwrap().count();
    println!("{} channels; sample rate: {}", channels_count, sample_rate);
    println!("Starting Processing");
    let audio_player = std::sync::Arc::new(std::sync::Mutex::new(AudioPlayer::new(samples, sample_rate, channels_count)));
    // player::play(
    //     AudioPlayer::new(samples, sample_rate, channels_count),
    //     audio_subsystem,
    // );
    Ok(())
}

// fn app(terminal: &mut DefaultTerminal) {

// }
