mod engine;
use crate::engine::engine::AudioEngine;
use std::thread;
use std::time::Duration;
use std::io::{Write};


fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("--- Audio Engine Example ---");

    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");
    let path = r"D:\Downloads\Khalid - Better (Official Video).mp3";
    engine.load(path)?;

    engine.play()?;
    // engine.set_bass_boost(true);
    // engine.set_bass_intensity(100.0);
    println!("Playback started. Press Enter to stop...");

    // Main loop to keep the engine ticking and handle device changes
    let mut input = String::new();
    loop {
        engine.tick();
        thread::sleep(Duration::from_millis(100));

        if engine.get_time_secs() > 120.0 {
            break;
        }
    }

    println!("Engine example finished.");
    Ok(())
}
