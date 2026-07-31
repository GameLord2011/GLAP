use std::{
    default::Default,
    fs::File,
    io::{Error as E, ErrorKind},
    thread::{JoinHandle, spawn},
};

use sdl2::{
    AudioSubsystem,
    audio::{AudioDevice, AudioSpecDesired},
};

use symphonia::{
    core::{
        codecs::audio::AudioDecoderOptions,
        errors::Error,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::{get_codecs, get_probe},
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

pub fn process_samples_from_file(path: String) -> JoinHandle<Result<AudioPlayer, E>> {
    spawn(move || {
        let ap: AudioPlayer;
        let file = Box::new(File::open(path).unwrap());
        let mss = MediaSourceStream::new(file, Default::default());

        let hint = Hint::new();

        let fmt_opts: FormatOptions = Default::default();
        let meta_opts: MetadataOptions = Default::default();
        let dec_opts: AudioDecoderOptions = Default::default();

        let format = get_probe().probe(&hint, mss, fmt_opts, meta_opts);

        if format.is_err() {
            return Err(E::other(
                format!("{}", format.err().unwrap()),
            ));
        } else {
            let mut good_format = format.unwrap();
            let track = good_format.default_track(TrackType::Audio).unwrap().clone();

            let decoder = get_codecs().make_audio_decoder(
                track.codec_params.as_ref().unwrap().audio().unwrap(),
                &dec_opts,
            );

            if decoder.is_err() {
                return Err(E::other(
                    format!("{}", decoder.err().unwrap()),
                ));
            } else {
                let mut good_decoder = decoder.unwrap();
                let track_id = track.id;
                let mut samples: Vec<f32> = Default::default();

                while let Some(packet) = good_format.next_packet().unwrap() {
                    if packet.track_id != track_id {
                        continue;
                    }

                    match good_decoder.decode(&packet) {
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

                let binding = track.codec_params.unwrap();
                let info = binding.audio().unwrap();
                let sample_rate = info.sample_rate.unwrap();
                let channels_count = info.channels.to_owned().unwrap().count();
                ap = AudioPlayer::new(samples, sample_rate, channels_count);
            }
        }
        Ok(ap)
    })
}
