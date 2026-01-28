mod engine;

use crate::engine::engine::AudioEngine;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Professional Audio Engine Example ---");

    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");

    let path = r"D:\Downloads\test eu\wow\Rehta hoon khud mein gum sa aksar.wav";
    
    if std::path::Path::new(path).exists() {
        println!("Loading: {}", path);
        engine.load(path)?;
        
        println!("Starting playback...");
        engine.play()?;

        thread::sleep(Duration::from_secs(30));
        
    } else {
        println!("Warning: Test file not found at: {}", path);
        println!("Please update the 'path' variable in src/main.rs to a valid audio file to see the engine in action.");
    }

    println!("Engine example finished.");
    Ok(())
}
