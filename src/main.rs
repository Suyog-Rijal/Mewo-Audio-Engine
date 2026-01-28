mod engine;

use crate::engine::engine::AudioEngine;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Professional Audio Engine Example ---");

    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");

    let path = r"D:\Downloads\Lauren Spencer-Smith - Fingers Crossed (Lyrics).mp3";
    
    if std::path::Path::new(path).exists() {
        println!("Loading: {}", path);
        engine.load(path)?;
        
        println!("Starting playback...");
        engine.play()?;

        // Simple playback loop for demonstration
        for i in 0..10 {
            let time = engine.get_time_secs();
            print!("\rPlaying: {:.2}s", time);
            io::stdout().flush()?;
            
            thread::sleep(Duration::from_millis(1000));

            if i == 4 {
                println!("\nPausing for 2 seconds...");
                engine.pause()?;
                thread::sleep(Duration::from_secs(5));
                println!("Resuming...");
                engine.play()?;
            }
        }
        
        println!("\nSeeking to 0.0s...");
        engine.seek(0.0);
        thread::sleep(Duration::from_secs(5));

        println!("Seeking to 65.0s...");
        engine.seek(65.0);
        thread::sleep(Duration::from_secs(10));

        println!("Stopping...");
        engine.stop();
    } else {
        println!("Warning: Test file not found at: {}", path);
        println!("Please update the 'path' variable in src/main.rs to a valid audio file to see the engine in action.");
    }

    println!("Engine example finished.");
    Ok(())
}
