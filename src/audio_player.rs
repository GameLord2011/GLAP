use sdl2::audio::AudioCallback;

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
    pub play: bool,
    pub finished: bool,
    whar_am_i: usize,
}

impl AudioPlayer {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: usize) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            play: true,
            finished: false,
            whar_am_i: 0,
        }
    }
}

impl AudioCallback for AudioPlayer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if !self.play {
            return;
        }
        let d = self.samples.len();
        if self.whar_am_i > d {
            for x in out.iter_mut() {
                *x = 0_f32
            }
        } else {
            let l = out.len();
            let next = self.whar_am_i + l;
            if next > d {
                let can_copy = l - (next - d);
                out[..can_copy].copy_from_slice(&self.samples[self.whar_am_i..]);
                self.whar_am_i += l;
                for i in out[can_copy..].iter_mut() {
                    *i = 0_f32
                }
            } else {
                out.copy_from_slice(&self.samples[self.whar_am_i..self.whar_am_i + l]);
                self.whar_am_i += l;
            }
        }
    }
}
