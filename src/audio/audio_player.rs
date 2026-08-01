use sdl2::audio::AudioCallback;
use std::fmt::{Debug, Result};

/**
    An Audio Player for use with SDL2.
   * samples: vector of 32-bit floating-point raw PCM samples (values from -1 to 1)
   * sample_rate: the rate of sampling in hertz (usually 44100)
   * channels: the number of channels
   * play: wether or not the audio is playing *IMPORTANT*: do not assign this value by default
   * whar_am_i: internal state tracking field
*/
pub struct AudioPlayer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
    pub finished: bool,
    whar_am_i: usize,
}

impl AudioPlayer {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: usize) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            finished: false,
            whar_am_i: 0,
        }
    }

    pub fn get_progress(&self) -> f64 {
        if self.finished {
            return 1_f64;
        }
        self.whar_am_i as f64 / self.samples.len() as f64
    }

    pub fn forward_2s(&mut self) {
        let forward = (self.sample_rate * 2) as usize;
        if self.whar_am_i + forward > self.samples.len() {
            self.finished = true;
        } else {
            self.whar_am_i += forward;
        }
    }

    pub fn back_2s(&mut self) {
        let backward = (self.sample_rate * 2) as usize;
        if let Some(d) = self.whar_am_i.checked_sub(backward) {
            self.whar_am_i = d;
        } else {
            self.whar_am_i = 0;
        }
    }

    pub fn restart(&mut self) {
        self.whar_am_i = 0;
        self.finished = false;
    }
}

// Basically for if an error occurs that makes it to where the contents of AudioPlayer
// are displayed it doesn't dump the entire contents of the samples vec to wherever.
impl Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        f.debug_struct("AudioPlayer")
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("finished", &self.finished)
            .field("whar_am_i", &self.whar_am_i)
            .finish()
    }
}

impl AudioCallback for AudioPlayer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let samples_len = self.samples.len();
        if self.finished {
            // Basically a fallback thing for a likely non-existent edge-case.
            for x in out.iter_mut() {
                *x = 0_f32
            }
        } else {
            let l = out.len();

            if (self.whar_am_i + l) > samples_len {
                // What it can copy.
                let can_copy = samples_len - self.whar_am_i;

                // Copies the rest of the existing buffer.
                out[..can_copy].copy_from_slice(&self.samples[self.whar_am_i..]);

                // Zeroes out the rest.
                for x in out[can_copy..].iter_mut() {
                    *x = 0_f32
                }

                // Sets that the song is done.
                self.finished = true;
            } else {
                // Normal Operation.
                out.copy_from_slice(&self.samples[self.whar_am_i..self.whar_am_i + l]);
                self.whar_am_i += l;
            }
        }
    }
}
