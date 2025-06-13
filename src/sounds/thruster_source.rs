use rodio::Source;
use std::time::Duration;
use std::f32::consts::PI;

pub struct ThrusterSource {
    sample_rate: u32,
    phase: f32,
    mod_phase: f32,
    active: bool,
    volume: f32,
}

impl ThrusterSource {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            phase: 0.0,
            mod_phase: 0.0,
            active: false,
            volume: 0.0,
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    fn generate_sample(&mut self) -> f32 {
        // Smooth volume ramping
        let target_volume = if self.active { 0.8 } else { 0.0 };
        self.volume += (target_volume - self.volume) * 0.01;
        
        // Debug every ~1 second
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 44100 == 0 {
                println!("ThrusterSource: active={}, volume={}", self.active, self.volume);
            }
        }
        
        if self.volume < 0.001 {
            return 0.0;
        }
        
        // Deep engine rumble
        let base_freq = 80.0;
        let base = (self.phase * base_freq * 2.0 * PI).sin() * 0.4;
        
        // Sub-bass for power
        let sub_freq = 40.0;
        let sub = (self.phase * sub_freq * 2.0 * PI).sin() * 0.3;
        
        // Mid frequency for body
        let mid_freq = 120.0;
        let mid = (self.phase * mid_freq * 2.0 * PI).sin() * 0.2;
        
        // Slow modulation for engine wobble
        let mod_freq = 5.0;
        let modulation = (self.mod_phase * mod_freq * 2.0 * PI).sin() * 0.1 + 1.0;
        
        // Update phases
        let sample_period = 1.0 / self.sample_rate as f32;
        self.phase += sample_period;
        self.mod_phase += sample_period;
        
        // Keep phases in reasonable range to avoid precision issues
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
        if self.mod_phase > 1.0 {
            self.mod_phase -= 1.0;
        }
        
        let output = (base + sub + mid) * modulation * self.volume * 2.0; // Boost volume
        output.clamp(-1.0, 1.0)
    }
}

impl Iterator for ThrusterSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate_sample())
    }
}

impl Source for ThrusterSource {
    fn current_frame_len(&self) -> Option<usize> {
        None // Infinite source
    }
    
    fn channels(&self) -> u16 {
        1 // Mono
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<Duration> {
        None // Infinite source
    }
}