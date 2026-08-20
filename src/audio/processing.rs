use std::{
    default::Default, fs::File, io::Error as E, thread::{JoinHandle, spawn},
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
) -> AudioDevice<AudioPlayer> {
    subsystem
        .open_playback(
            None,
            &AudioSpecDesired {
                freq: Some(audio_player.sample_rate as i32),
                channels: Some(audio_player.channels as u8),
                samples: None,
            },
            |_unused_spec| audio_player, // Screw the wanted audio spec lol.
        )
        .unwrap()
}

pub fn process_samples_from_file(path: String) -> JoinHandle<Result<AudioPlayer, E>> {
    spawn(move || {
        let format = get_probe().probe(
            &Hint::new(),
            MediaSourceStream::new(Box::new(File::open(path).unwrap()), Default::default()),
            FormatOptions::default(),
            MetadataOptions::default(),
        );

        if let Ok(mut good_format) = format {
            let track = good_format.default_track(TrackType::Audio).unwrap().clone();

            let decoder = get_codecs().make_audio_decoder(
                track.codec_params.as_ref().unwrap().audio().unwrap(),
                &AudioDecoderOptions::default(),
            );

            if let Ok(mut good_decoder) = decoder {
                let mut samples: Vec<f32> = Vec::new();

                while let Some(packet) = good_format.next_packet().unwrap() {
                    if packet.track_id != track.id {
                        continue;
                    }

                    match good_decoder.decode(&packet) {
                        Ok(audio_buf) => {
                            let mut t: Vec<f32> = Vec::new();
                            t.resize(audio_buf.samples_interleaved(), 0_f32);
                            audio_buf.copy_to_slice_interleaved(&mut t);
                            samples.append(&mut t);
                        }
                        Err(Error::DecodeError(_)) => (), // Symphonia says it's fine.
                        Err(_) => break,
                    }
                }

                // Why do I need a binding here :sob:
                let binding = track.codec_params.unwrap();
                let info = binding.audio().unwrap();
                Ok(AudioPlayer::new(
                    samples,
                    info.sample_rate.unwrap(),
                    info.channels.to_owned().unwrap().count(),
                ))
            } else {
                Err(E::other(format!("{}", decoder.err().unwrap())))
            }
        } else {
            Err(E::other(format!("{}", format.err().unwrap())))
        }
    })
}
