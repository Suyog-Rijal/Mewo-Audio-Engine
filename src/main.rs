mod engine;

use crate::engine::engine::AudioEngine;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Professional Audio Engine Example ---");

    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");

    let path = r"D:\Downloads\Rehta hoon khud mein gum sa aksar.mp3";
    engine.load(path)?;
    println!("Audio file loaded: {}", path);
    engine.seek(70.0);
    engine.play()?;
    println!("Playback started.");
    
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(120) {
        engine.tick();
        thread::sleep(Duration::from_millis(100));
    }

    println!("Engine example finished.");
    Ok(())
}
