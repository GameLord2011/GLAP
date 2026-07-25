use sdl2::{
    AudioSubsystem,
    audio::{AudioDevice, AudioSpecDesired},
};

use crate::audio_player::AudioPlayer;

pub fn play(
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
