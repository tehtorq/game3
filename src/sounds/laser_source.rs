use rodio::Source;
use std::time::Duration;
use std::f32::consts::PI;

pub enum LaserType {
    PlayerLaser,
    EnemyPulse,
    EnemyBeam,
    EnemyPlasma,
}

pub struct LaserSource {
    sample_rate: u32,
    time: f32,
    duration: f32,
    laser_type: LaserType,
    finished: bool,
}

impl LaserSource {
    pub fn new(laser_type: LaserType) -> Self {
        let duration = match laser_type {
            LaserType::PlayerLaser => 0.15,
            LaserType::EnemyPulse => 0.1,
            LaserType::EnemyBeam => 0.3,
            LaserType::EnemyPlasma => 0.2,
        };
        
        Self {
            sample_rate: 44100,
            time: 0.0,
            duration,
            laser_type,
            finished: false,
        }
    }
    
    fn generate_sample(&mut self) -> f32 {
        if self.finished || self.time > self.duration {
            self.finished = true;
            return 0.0;
        }
        
        let t = self.time;
        let progress = t / self.duration;
        
        let sample = match self.laser_type {
            LaserType::PlayerLaser => {
                // Deep, punchy laser sound - 30% lower pitch
                let freq = 560.0 * (1.0 - progress * 0.6); // Sweep from 560Hz to 224Hz (30% lower)
                let carrier = (t * freq * 2.0 * PI).sin();
                // Add a sub-harmonic for punch
                let sub_freq = freq * 0.5;
                let sub_carrier = (t * sub_freq * 2.0 * PI).sin() * 0.3;
                let envelope = (1.0 - progress).powf(0.5); // Quick decay
                let noise = ((t * 12345.6).sin() * 43758.5453).fract() * 0.05;
                (carrier + sub_carrier + noise) * envelope * 0.35
            }
            
            LaserType::EnemyPulse => {
                // Short pulse burst
                let freq = 800.0;
                let carrier = (t * freq * 2.0 * PI).sin();
                let pulse = ((t * 50.0 * 2.0 * PI).sin() * 0.5 + 0.5).powf(3.0);
                let envelope = (1.0 - progress).powf(2.0);
                carrier * pulse * envelope * 0.2
            }
            
            LaserType::EnemyBeam => {
                // Sustained beam with wobble
                let base_freq = 150.0;
                let wobble = (t * 10.0 * 2.0 * PI).sin() * 20.0;
                let freq = base_freq + wobble;
                let carrier = (t * freq * 2.0 * PI).sin();
                let envelope = if progress < 0.1 {
                    progress * 10.0 // Fade in
                } else if progress > 0.8 {
                    (1.0 - progress) * 5.0 // Fade out
                } else {
                    1.0
                };
                carrier * envelope * 0.15
            }
            
            LaserType::EnemyPlasma => {
                // Warbling plasma sound
                let freq1 = 600.0 + (t * 15.0 * 2.0 * PI).sin() * 100.0;
                let freq2 = 900.0 + (t * 12.0 * 2.0 * PI).sin() * 150.0;
                let carrier1 = (t * freq1 * 2.0 * PI).sin() * 0.5;
                let carrier2 = (t * freq2 * 2.0 * PI).sin() * 0.3;
                let envelope = (1.0 - progress).powf(1.5);
                (carrier1 + carrier2) * envelope * 0.2
            }
        };
        
        self.time += 1.0 / self.sample_rate as f32;
        sample.clamp(-1.0, 1.0)
    }
}

impl Iterator for LaserSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            Some(self.generate_sample())
        }
    }
}

impl Source for LaserSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    
    fn channels(&self) -> u16 {
        1 // Mono
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.duration))
    }
}