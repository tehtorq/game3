use rodio::Source;
use std::time::Duration;
use std::f32::consts::PI;

pub struct MusicSource {
    sample_rate: u32,
    time: f32,
    tempo: f32, // BPM
    music_state: crate::sounds::MusicState,
    previous_state: crate::sounds::MusicState,
    state_transition: f32, // 0.0 = previous state, 1.0 = current state
}

impl MusicSource {
    pub fn new(music_state: crate::sounds::MusicState) -> Self {
        let tempo = match music_state {
            crate::sounds::MusicState::Peaceful => 80.0,
            crate::sounds::MusicState::Suspense => 100.0,
            crate::sounds::MusicState::Combat => 120.0,
        };
        
        Self {
            sample_rate: 44100,
            time: 0.0,
            tempo,
            music_state,
            previous_state: music_state,
            state_transition: 1.0,
        }
    }
    
    pub fn set_state(&mut self, state: crate::sounds::MusicState) {
        if self.music_state != state {
            self.previous_state = self.music_state;
            self.music_state = state;
            self.state_transition = 0.0; // Start transition
            self.tempo = match state {
                crate::sounds::MusicState::Peaceful => 80.0,
                crate::sounds::MusicState::Suspense => 100.0,
                crate::sounds::MusicState::Combat => 120.0,
            };
        }
    }
    
    fn generate_sample(&mut self) -> f32 {
        let beat_duration = 60.0 / self.tempo;
        let beat_pos = (self.time % beat_duration) / beat_duration;
        let bar_pos = (self.time % (beat_duration * 4.0)) / (beat_duration * 4.0);
        let phrase_pos = (self.time % (beat_duration * 16.0)) / (beat_duration * 16.0);
        
        // Smooth loop transition
        let loop_fade = if phrase_pos > 0.98 {
            (1.0 - phrase_pos) * 50.0  // Fade out at end of phrase
        } else if phrase_pos < 0.02 {
            phrase_pos * 50.0  // Fade in at start of phrase
        } else {
            1.0
        };
        
        // Update state transition
        if self.state_transition < 1.0 {
            self.state_transition = (self.state_transition + 1.0 / self.sample_rate as f32 * 0.5).min(1.0); // 2 second transition
        }
        
        let mut output = 0.0;
        let mut previous_output = 0.0;
        
        // Generate output for current state
        match self.music_state {
            crate::sounds::MusicState::Peaceful => {
                // Simple ambient pad with more movement - 30% lower
                let pad_freq = 77.0; // Between E2 and F2 (30% lower than A2)
                let pad = (self.time * pad_freq * 2.0 * PI).sin() * 0.1;
                let pad_fifth = (self.time * pad_freq * 1.5 * 2.0 * PI).sin() * 0.05;
                
                // Add a slow LFO for movement
                let lfo = (self.time * 0.3).sin() * 0.02;
                let pad_octave = (self.time * pad_freq * 4.0 * 2.0 * PI).sin() * lfo;
                
                output += pad + pad_fifth + pad_octave;
            }
            
            crate::sounds::MusicState::Suspense => {
                // Classic suspense pattern: low bass note with tension - 30% lower
                // Bass line - alternating root and fifth
                let bass_freq = if (bar_pos * 8.0) as i32 % 2 == 0 { 38.5 } else { 57.75 }; // 30% lower
                let bass_env = if beat_pos < 0.1 { 1.0 } else { (1.0 - beat_pos).powf(2.0) };
                let bass = (self.time * bass_freq * 2.0 * PI).sin() * bass_env * 0.3;
                
                // High tension notes - minor second intervals - 30% lower
                let tension_freq = 308.0 * (1.0 + (phrase_pos * PI * 2.0).sin() * 0.06); // 30% lower than 440
                let tension_pattern = ((beat_pos * 8.0) as i32 % 4) as f32;
                let tension_env = if tension_pattern == 0.0 || tension_pattern == 2.0 { 0.2 } else { 0.0 };
                let tension = (self.time * tension_freq * 2.0 * PI).sin() * tension_env;
                
                // Add some filtered noise for atmosphere
                let noise = ((self.time * 12345.6).sin() * 43758.5453).fract() * 0.02;
                
                // Add a low drone for continuity - 30% lower
                let drone_freq = 19.25; // 30% lower than A0
                let drone = (self.time * drone_freq * 2.0 * PI).sin() * 0.1;
                
                output = bass + tension + noise + drone;
            }
            
            crate::sounds::MusicState::Combat => {
                // Fast-paced combat music - 30% lower
                // Driving bass line
                let bass_pattern = [38.5, 38.5, 57.75, 51.38]; // 30% lower
                let bass_idx = (beat_pos * 4.0) as usize % 4;
                let bass_freq = bass_pattern[bass_idx];
                let bass_env = (1.0 - (beat_pos * 4.0).fract()).powf(3.0);
                let bass = (self.time * bass_freq * 2.0 * PI).sin() * bass_env * 0.4;
                
                // Arpeggiated lead - 30% lower
                let lead_pattern = [308.0, 366.28, 461.48, 366.28]; // 30% lower
                let lead_idx = (beat_pos * 16.0) as usize % 4;
                let lead_freq = lead_pattern[lead_idx];
                let lead_env = 0.15;
                let lead = ((self.time * lead_freq * 2.0 * PI).sin() * 0.5 + 
                           (self.time * lead_freq * 2.0 * 2.0 * PI).sin() * 0.3) * lead_env;
                
                // Rhythmic noise hits
                let hit_env = if (beat_pos * 4.0).fract() < 0.05 { 1.0 } else { 0.0 };
                let hit = ((self.time * 140.0).sin() * noise_gen(self.time)) * hit_env * 0.2; // 30% lower
                
                // Add a continuous pad for smoother sound - 30% lower
                let pad_freq = 154.0; // 30% lower than A3
                let combat_pad = (self.time * pad_freq * 2.0 * PI).sin() * 0.05;
                
                output = bass + lead + hit + combat_pad;
            }
        }
        
        // Generate output for previous state if transitioning
        if self.state_transition < 1.0 {
            match self.previous_state {
                crate::sounds::MusicState::Peaceful => {
                    let pad_freq = 77.0; // 30% lower
                    let pad = (self.time * pad_freq * 2.0 * PI).sin() * 0.1;
                    let pad_fifth = (self.time * pad_freq * 1.5 * 2.0 * PI).sin() * 0.05;
                    let lfo = (self.time * 0.3).sin() * 0.02;
                    let pad_octave = (self.time * pad_freq * 4.0 * 2.0 * PI).sin() * lfo;
                    previous_output = pad + pad_fifth + pad_octave;
                }
                crate::sounds::MusicState::Suspense => {
                    let bass_freq = if (bar_pos * 8.0) as i32 % 2 == 0 { 38.5 } else { 57.75 }; // 30% lower
                    let bass_env = if beat_pos < 0.1 { 1.0 } else { (1.0 - beat_pos).powf(2.0) };
                    let bass = (self.time * bass_freq * 2.0 * PI).sin() * bass_env * 0.3;
                    let tension_freq = 308.0 * (1.0 + (phrase_pos * PI * 2.0).sin() * 0.06); // 30% lower
                    let tension_pattern = ((beat_pos * 8.0) as i32 % 4) as f32;
                    let tension_env = if tension_pattern == 0.0 || tension_pattern == 2.0 { 0.2 } else { 0.0 };
                    let tension = (self.time * tension_freq * 2.0 * PI).sin() * tension_env;
                    let noise = ((self.time * 12345.6).sin() * 43758.5453).fract() * 0.02;
                    let drone = (self.time * 19.25 * 2.0 * PI).sin() * 0.1; // 30% lower
                    previous_output = bass + tension + noise + drone;
                }
                crate::sounds::MusicState::Combat => {
                    let bass_pattern = [38.5, 38.5, 57.75, 51.38]; // 30% lower
                    let bass_idx = (beat_pos * 4.0) as usize % 4;
                    let bass_freq = bass_pattern[bass_idx];
                    let bass_env = (1.0 - (beat_pos * 4.0).fract()).powf(3.0);
                    let bass = (self.time * bass_freq * 2.0 * PI).sin() * bass_env * 0.4;
                    let lead_pattern = [308.0, 366.28, 461.48, 366.28]; // 30% lower
                    let lead_idx = (beat_pos * 16.0) as usize % 4;
                    let lead_freq = lead_pattern[lead_idx];
                    let lead = ((self.time * lead_freq * 2.0 * PI).sin() * 0.5 + 
                               (self.time * lead_freq * 2.0 * 2.0 * PI).sin() * 0.3) * 0.15;
                    let hit_env = if (beat_pos * 4.0).fract() < 0.05 { 1.0 } else { 0.0 };
                    let hit = ((self.time * 140.0).sin() * noise_gen(self.time)) * hit_env * 0.2; // 30% lower
                    let combat_pad = (self.time * 154.0 * 2.0 * PI).sin() * 0.05; // 30% lower
                    previous_output = bass + lead + hit + combat_pad;
                }
            }
            
            // Crossfade between previous and current state
            output = previous_output * (1.0 - self.state_transition) + output * self.state_transition;
        }
        
        self.time += 1.0 / self.sample_rate as f32;
        
        // Apply loop fade and soft clipping for warmth
        output.tanh() * 0.5 * loop_fade
    }
}

fn noise_gen(t: f32) -> f32 {
    ((t * 12345.6).sin() * 43758.5453).fract() * 2.0 - 1.0
}

impl Iterator for MusicSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate_sample())
    }
}

impl Source for MusicSource {
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
        None // Infinite
    }
}