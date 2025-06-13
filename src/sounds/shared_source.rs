use rodio::Source;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SharedThrusterSource {
    inner: Arc<Mutex<super::thruster_source::ThrusterSource>>,
}

impl SharedThrusterSource {
    pub fn new(inner: Arc<Mutex<super::thruster_source::ThrusterSource>>) -> Self {
        Self { inner }
    }
}

impl Iterator for SharedThrusterSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(mut source) = self.inner.lock() {
            source.next()
        } else {
            Some(0.0) // Return silence if we can't lock
        }
    }
}

impl Source for SharedThrusterSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    
    fn channels(&self) -> u16 {
        1 // Mono - we'll convert to stereo in the mod.rs
    }
    
    fn sample_rate(&self) -> u32 {
        44100
    }
    
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// Shared source for music
pub struct SharedMusicSource {
    inner: Arc<Mutex<super::music_source::MusicSource>>,
}

impl SharedMusicSource {
    pub fn new(inner: Arc<Mutex<super::music_source::MusicSource>>) -> Self {
        Self { inner }
    }
}

impl Iterator for SharedMusicSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(mut source) = self.inner.lock() {
            source.next()
        } else {
            Some(0.0) // Return silence if we can't lock
        }
    }
}

impl Source for SharedMusicSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    
    fn channels(&self) -> u16 {
        1 // Mono - we'll convert to stereo
    }
    
    fn sample_rate(&self) -> u32 {
        44100
    }
    
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}