extern crate sdl2;

use std::{io::stdin, path::Path, time::Duration};

use creak::Decoder;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

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

    let samples = Decoder::open(Path::new(&path)).unwrap();
    let info = samples.info();

    println!(
        "{}; {}; {}",
        info.channels(),
        info.format(),
        info.sample_rate()
    );
    let desired_spec = AudioSpecDesired {
        freq: Some(info.sample_rate() as i32),
        channels: Some(info.channels() as u8),
        samples: None,
    };
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| AudioPlayer {
            samples: samples
                .into_samples()
                .unwrap()
                .map(|s| s.unwrap())
                .collect::<Vec<f32>>(),
            whar_am_i: 0,
        })
        .unwrap();

    device.resume();
    std::thread::sleep(Duration::from_secs(50));
}
