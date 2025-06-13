use rodio::{OutputStream, source::SineWave, Source, Sink};
use std::time::Duration;

fn main() {
    println!("Testing rodio audio...");
    
    // Try to create output stream
    match OutputStream::try_default() {
        Ok((_stream, stream_handle)) => {
            println!("Audio device opened successfully");
            
            // Create a sink
            match Sink::try_new(&stream_handle) {
                Ok(sink) => {
                    println!("Sink created");
                    
                    // Create a 440Hz sine wave
                    let source = SineWave::new(440.0)
                        .take_duration(Duration::from_secs(2))
                        .amplify(0.5);
                    
                    // Play it
                    sink.append(source);
                    sink.set_volume(1.0);
                    println!("Playing 440Hz tone for 2 seconds...");
                    
                    // Sleep while it plays
                    sink.sleep_until_end();
                    println!("Done!");
                }
                Err(e) => println!("Failed to create sink: {}", e),
            }
        }
        Err(e) => println!("Failed to open audio device: {}", e),
    }
}