use std::{default::Default, fs::File};

use sdl2::{
    AudioSubsystem,
    audio::{AudioDevice, AudioSpecDesired},
};

use symphonia::core::{
    codecs::audio::AudioDecoderOptions,
    errors::Error,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

use crate::audio::audio_player::AudioPlayer;

pub fn create_device(
    audio_player: AudioPlayer,
    subsystem: AudioSubsystem,
) -> (f64, AudioDevice<AudioPlayer>) {
    let desired_spec = AudioSpecDesired {
        freq: Some(audio_player.sample_rate as i32),
        channels: Some(audio_player.channels as u8),
        samples: None,
    };

    let secs = (audio_player.samples.len() as f64 / audio_player.channels as f64)
        / audio_player.sample_rate as f64;
    let device = subsystem
        .open_playback(None, &desired_spec, |_| audio_player)
        .unwrap();
    (secs, device)
}

pub fn process_samples_from_file(path: String, volume: f32) -> AudioPlayer {
    let d: std::thread::JoinHandle<AudioPlayer> = std::thread::spawn(move || {
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

        // #[cfg(debug_assertions)]
        // {
        //     println!(
        //         "Starting audio sample parsing. This can take a fat minute in dev so be patient."
        //     );
        // }
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
        // println!("{:?}", samples.len());

        let binding = track.codec_params.unwrap();
        let info = binding.audio().unwrap();
        let sample_rate = info.sample_rate.unwrap();
        let channels_count = info.channels.to_owned().unwrap().count();
        // println!("{} channels; sample rate: {}", channels_count, sample_rate);
        // println!("Starting Processing");
        AudioPlayer::new(samples, sample_rate, channels_count, volume).unwrap()
    });

    d.join().unwrap()
}
