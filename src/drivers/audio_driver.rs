use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Standalone struct for the audio driver
pub struct AudioDriver {
    is_active: Arc<AtomicBool>,
}

impl AudioDriver {
    pub fn new(
        audio_subsystem: &sdl2::AudioSubsystem,
        frequency: f32,
        volume: f32,
    ) -> Result<Self, String> {
        let is_active = Arc::new(AtomicBool::new(false));
        let is_active_clone = is_active.clone();

        let desired_spec = AudioSpecDesired {
            freq: None,          // Let SDL2 decide the best frequency
            channels: None,      // Let SDL2 decide the best channel count
            samples: Some(1024), // Larger buffer size for stability            samples: Some(1024),
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, move |spec| SineWave {
            phase_inc: frequency * 2.0 * PI / spec.freq as f32,
            phase: 0.0,
            volume,
            is_active: is_active_clone,
        })?;

        println!(
            "Actual AudioSpec: freq = {}, channels = {}, samples = {}",
            device.spec().freq,
            device.spec().channels,
            device.spec().samples
        );

        device.resume();

        Ok(Self { is_active })
    }

    pub fn toggle(&self) {
        let current_state = self.is_active.load(Ordering::Relaxed);
        self.is_active.store(!current_state, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }
}

// Struct for the audio callback
struct SineWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    is_active: Arc<AtomicBool>,
}

impl AudioCallback for SineWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            if self.is_active.load(Ordering::Relaxed) {
                *sample = self.phase.sin() * self.volume;
                self.phase = (self.phase + self.phase_inc) % (2.0 * PI);
            } else {
                *sample = 0.0;
            }
        }
    }
}
