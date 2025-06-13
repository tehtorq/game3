use std::f32::consts::PI;
#[cfg(feature = "audio")]
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
#[cfg(feature = "audio")]
use std::sync::{Arc, Mutex};

pub mod thruster;
pub use thruster::ThrusterSound;

#[cfg(feature = "audio")]
pub mod thruster_source;
#[cfg(feature = "audio")]
pub mod shared_source;
#[cfg(feature = "audio")]
pub mod laser_source;
#[cfg(feature = "audio")]
pub mod music_source;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicState {
    Peaceful,
    Suspense,
    Combat,
}

pub struct SoundSystem {
    // Music state
    current_music_state: MusicState,
    music_transition: f32,
    target_music_state: MusicState,
    
    // Music parameters
    music_time: f32,
    base_tempo: f32, // BPM
    
    // Track if sounds are loaded
    initialized: bool,
    
    // Track thruster state
    thruster_playing: bool,
    
    // Audio system
    #[cfg(feature = "audio")]
    _stream: Option<OutputStream>,
    #[cfg(feature = "audio")]
    stream_handle: Option<OutputStreamHandle>,
    
    // Music system
    #[cfg(feature = "audio")]
    music_source: Option<Arc<Mutex<music_source::MusicSource>>>,
    #[cfg(feature = "audio")]
    music_sink: Option<Sink>,
    
    // Thruster sound
    #[cfg(feature = "audio")]
    thruster_source: Option<Arc<Mutex<thruster_source::ThrusterSource>>>,
    #[cfg(feature = "audio")]
    thruster_sink: Option<Sink>,
    
    // Sound limiting
    #[cfg(feature = "audio")]
    last_laser_time: std::time::Instant,
    #[cfg(feature = "audio")]
    active_sounds: u32,
}

impl SoundSystem {
    pub fn new() -> Self {
        Self {
            current_music_state: MusicState::Peaceful,
            music_transition: 1.0,
            target_music_state: MusicState::Peaceful,
            music_time: 0.0,
            base_tempo: 80.0,
            initialized: false,
            thruster_playing: false,
            #[cfg(feature = "audio")]
            _stream: None,
            #[cfg(feature = "audio")]
            stream_handle: None,
            #[cfg(feature = "audio")]
            thruster_source: None,
            #[cfg(feature = "audio")]
            thruster_sink: None,
            #[cfg(feature = "audio")]
            music_source: None,
            #[cfg(feature = "audio")]
            music_sink: None,
            #[cfg(feature = "audio")]
            last_laser_time: std::time::Instant::now(),
            #[cfg(feature = "audio")]
            active_sounds: 0,
        }
    }
    
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }
        
        #[cfg(feature = "audio")]
        {
            // Initialize rodio
            match OutputStream::try_default() {
                Ok((stream, stream_handle)) => {
                    println!("Audio device opened successfully");
                    self._stream = Some(stream);
                    self.stream_handle = Some(stream_handle);
                    
                    // Create thruster source
                    let thruster_source = Arc::new(Mutex::new(thruster_source::ThrusterSource::new()));
                    self.thruster_source = Some(thruster_source.clone());
                    
                    // Create sink and attach the source
                    if let Some(handle) = &self.stream_handle {
                        match Sink::try_new(handle) {
                            Ok(sink) => {
                                // Create a shared source wrapper and convert to stereo
                                let shared_source = shared_source::SharedThrusterSource::new(thruster_source.clone());
                                let stereo_source = shared_source.convert_samples::<f32>();
                                sink.append(stereo_source);
                                sink.set_volume(1.0);
                                println!("Sink created, volume set to 1.0, source attached");
                                // The source starts silent until activated
                                self.thruster_sink = Some(sink);
                            }
                            Err(e) => {
                                println!("Failed to create sink: {}", e);
                            }
                        }
                    }
                    
                    // Create music source and sink
                    let music_source = Arc::new(Mutex::new(music_source::MusicSource::new(MusicState::Peaceful)));
                    self.music_source = Some(music_source.clone());
                    
                    if let Some(handle) = &self.stream_handle {
                        match Sink::try_new(handle) {
                            Ok(sink) => {
                                // Create a shared source wrapper for music
                                let shared_music = shared_source::SharedMusicSource::new(music_source.clone());
                                let stereo_music = shared_music.convert_samples::<f32>();
                                sink.append(stereo_music);
                                sink.set_volume(0.4); // Background music volume
                                self.music_sink = Some(sink);
                                println!("Music system initialized");
                            }
                            Err(e) => {
                                println!("Failed to create music sink: {}", e);
                            }
                        }
                    }
                    
                    println!("Sound system initialized with rodio");
                }
                Err(e) => {
                    println!("Failed to initialize audio: {}", e);
                }
            }
        }
        
        self.initialized = true;
    }
    
    // Music generation
    pub fn update_music(&mut self, state: MusicState, dt: f32) {
        self.target_music_state = state;
        self.music_time += dt;
        
        // Smooth transition
        if self.target_music_state != self.current_music_state {
            self.music_transition -= dt * 0.5; // 2 second transition
            if self.music_transition <= 0.0 {
                self.current_music_state = self.target_music_state;
                self.music_transition = 0.0;
                
                // Update music source state
                #[cfg(feature = "audio")]
                {
                    if let Some(source) = &self.music_source {
                        if let Ok(mut src) = source.lock() {
                            src.set_state(self.current_music_state);
                        }
                    }
                }
            }
        } else {
            self.music_transition = (self.music_transition + dt * 0.5).min(1.0);
        }
        
        // Update tempo based on state
        let target_tempo = match self.target_music_state {
            MusicState::Peaceful => 80.0,
            MusicState::Suspense => 120.0,
            MusicState::Combat => 160.0,
        };
        
        self.base_tempo = self.base_tempo + (target_tempo - self.base_tempo) * dt * 0.5;
    }
    
    // Public interface
    pub fn play_laser(&self) {
        if self.initialized {
            #[cfg(feature = "audio")]
            {
                if let Some(handle) = &self.stream_handle {
                    let source = laser_source::LaserSource::new(laser_source::LaserType::PlayerLaser);
                    let _ = handle.play_raw(source.convert_samples());
                }
            }
        }
    }
    
    pub fn play_enemy_laser(&mut self, enemy_type: &str, distance: Option<f32>) {
        if self.initialized {
            #[cfg(feature = "audio")]
            {
                // Limit sounds to prevent audio overload
                const MAX_SOUNDS_PER_FRAME: u32 = 5;
                const MIN_SOUND_INTERVAL_MS: u128 = 30; // Minimum 30ms between sounds
                const MAX_AUDIBLE_DISTANCE: f32 = 2000.0; // Sounds beyond this are not played
                const CLOSE_DISTANCE: f32 = 500.0; // Sounds within this always play
                
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(self.last_laser_time).as_millis();
                
                // Check distance if provided
                if let Some(dist) = distance {
                    // Don't play sounds that are too far away
                    if dist > MAX_AUDIBLE_DISTANCE {
                        return;
                    }
                    
                    // Always play close sounds
                    if dist > CLOSE_DISTANCE {
                        // For distant sounds, apply stricter limiting
                        if self.active_sounds >= MAX_SOUNDS_PER_FRAME || elapsed < MIN_SOUND_INTERVAL_MS {
                            return;
                        }
                    }
                }
                
                // Check if this is a base turret (they use string names directly)
                // Actual enemies come through as "EnemyType::Name" format
                let is_turret = !enemy_type.contains("::");
                
                // For turrets, apply additional limiting based on distance
                if is_turret && distance.unwrap_or(0.0) > CLOSE_DISTANCE {
                    if self.active_sounds >= 2 { // Strict limit for distant turrets
                        return;
                    }
                }
                
                if let Some(handle) = &self.stream_handle {
                    let laser_type = match enemy_type {
                        "Vortex" | "Swarm" => laser_source::LaserType::EnemyPulse,
                        "Guardian" | "Carrier" => laser_source::LaserType::EnemyBeam,
                        "Reflector" => laser_source::LaserType::EnemyPlasma,
                        _ => laser_source::LaserType::EnemyPulse,
                    };
                    let source = laser_source::LaserSource::new(laser_type);
                    
                    // Calculate volume based on distance
                    let volume = if let Some(dist) = distance {
                        // Linear falloff from 1.0 at 0 distance to 0.1 at MAX_AUDIBLE_DISTANCE
                        let volume_scale = 1.0 - (dist / MAX_AUDIBLE_DISTANCE * 0.9);
                        volume_scale.clamp(0.1, 1.0)
                    } else {
                        1.0
                    };
                    
                    // Apply volume by amplifying the source
                    let source = source.amplify(volume);
                    
                    if handle.play_raw(source.convert_samples()).is_ok() {
                        self.active_sounds += 1;
                        self.last_laser_time = now;
                    }
                }
            }
        }
    }
    
    pub fn update(&mut self, _dt: f32) {
        // Reset sound counter each frame
        #[cfg(feature = "audio")]
        {
            self.active_sounds = 0;
        }
    }
    
    pub fn play_explosion(&self) {
        // Placeholder - will add actual sound later
        if self.initialized {
            // println!("Boom!"); // Uncomment for debug
        }
    }
    
    pub fn set_volume(&self, _volume: f32) {
        // Placeholder
    }
    
    pub fn update_thruster(&mut self, is_thrusting: bool) {
        // Only update if state changed
        if is_thrusting != self.thruster_playing {
            self.thruster_playing = is_thrusting;
            
            if self.initialized {
                #[cfg(feature = "audio")]
                {
                    // Update the thruster source state
                    if let Some(source) = &self.thruster_source {
                        if let Ok(mut src) = source.lock() {
                            src.set_active(is_thrusting);
                        }
                    }
                    
                    if is_thrusting {
                        println!("Thruster: ON (with audio)");
                    } else {
                        println!("Thruster: OFF (with audio)");
                    }
                }
                
                #[cfg(not(feature = "audio"))]
                {
                    if is_thrusting {
                        println!("Thruster: ON (no audio)");
                    } else {
                        println!("Thruster: OFF (no audio)");
                    }
                }
            }
        }
    }
}