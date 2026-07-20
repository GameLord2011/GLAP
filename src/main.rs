extern crate sdl2;

use std::{default::Default, fs::File, io::stdin, time::Duration};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use symphonia::core::{
    codecs::audio::AudioDecoderOptions,
    errors::Error,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

enum Page {
    Home,
    About,
    Player,
    Settings,
}

struct AudioPlayer {
    samples: Vec<f32>,
    whar_am_i: usize,
}

impl AudioCallback for AudioPlayer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let d = self.samples.len();
        if self.whar_am_i > d {
            for x in out.iter_mut() {
                *x = 0_f32
            }
        } else {
            let l = out.len();
            let next = self.whar_am_i + l;
            if next > d {
                let left = next - d;
                let can_copy = out.len() - left;
                out[..can_copy].copy_from_slice(&self.samples[self.whar_am_i..]);
                self.whar_am_i += l;
                for i in out[left..].iter_mut() {
                    *i = 0_f32
                }
            } else {
                out[..l].copy_from_slice(&self.samples[self.whar_am_i..self.whar_am_i + l]);
                self.whar_am_i += l;
            }
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut path = String::new();
    let p = std::env::args().nth(1);
    match p {
        Some(n) => path = n,
        None => {
            println!("Whar is the file:");
            stdin().read_line(&mut path).unwrap();
        }
    }
    path = path.trim_matches(['\r', '\n', '"', '\'']).to_owned();

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
    }

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

    while let Some(packet) = format.next_packet().unwrap() {
        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                samples.resize(audio_buf.samples_interleaved(), 0_f32); // Flood the vec with zeros(?);
                audio_buf.copy_to_slice_interleaved(&mut samples);
            }
            Err(Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }

    let binding = track.codec_params.unwrap();
    let info = binding.audio().unwrap();
    let sample_rate = info.sample_rate.unwrap();
    let channels_count = info.channels.to_owned().unwrap().count();
    println!(
        "{} channels; sample rate: {}",
        channels_count,
        sample_rate
    );
    println!("Starting Processing");
    let desired_spec = AudioSpecDesired {
        freq: Some(sample_rate as i32),
        channels: Some(channels_count as u8),
        samples: None,
    };
    println!("Desired Spec Built");
    println!("Samples Vector Built");
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| AudioPlayer {
            samples: samples.clone(),
            whar_am_i: 0,
        })
        .unwrap();
    println!("Device opened");
    println!("{}", samples.len());

    // let secs = (samples.len() as f64 / channels_count as f64) / sample_rate as f64;
    // let secs = binding.audio().unwrap().t
    // println!("{secs}");
    // device.resume();
    // println!("Playing!");
    // std::thread::sleep(Duration::from_secs_f64(secs));
    // std::thread::sleep(Duration::from_secs(2));
}
