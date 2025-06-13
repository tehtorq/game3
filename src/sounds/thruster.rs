use std::f32::consts::PI;

pub struct ThrusterSound {
    phase: f32,
    noise_phase: f32,
    noise_counter: f32,
    sample_rate: f32,
    volume: f32,
    warmup: f32,
}

impl ThrusterSound {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            noise_phase: 0.0,
            noise_counter: 0.0,
            sample_rate,
            volume: 0.0,
            warmup: 0.0,
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.volume = if active { 1.0 } else { 0.0 };
    }
    
    pub fn generate_sample(&mut self) -> f32 {
        if self.volume == 0.0 {
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
        let modulation = (self.phase * mod_freq * 2.0 * PI).sin() * 0.1 + 1.0;
        
        self.phase += 1.0 / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        
        let output = (base + sub + mid) * modulation * self.volume * 0.6;
        output.clamp(-1.0, 1.0)
    }
    
    fn generate_noise(&mut self) -> f32 {
        self.noise_counter += 1.0;
        let x = (self.noise_counter * 12.9898 + 78.233).sin() * 43758.5453;
        (x - x.floor()) * 2.0 - 1.0
    }
    
    pub fn generate_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.generate_sample();
        }
    }
}