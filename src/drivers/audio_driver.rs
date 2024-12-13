use sdl2::audio::{AudioCallback, AudioSpecDesired};

// Standalone struct for the audio driver
pub struct AudioDriver {
    device: sdl2::audio::AudioDevice<SquareWave>,
}

impl AudioDriver {
    pub fn new(sdl_context: &sdl2::Sdl, frequency: f32, volume: f32) -> Result<Self, String> {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: frequency / spec.freq as f32,
            phase: 0.0,
            volume,
        })?;

        device.resume();
        Ok(Self { device })
    }

    pub fn stop(&self) {
        self.device.pause();
    }

    pub fn start(&self) {
        self.device.resume();
    }

    pub fn is_active(&self) -> bool {
        self.device.status() == sdl2::audio::AudioStatus::Playing
    }
}

// Struct for the audio callback
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
