mod engine;
use crate::engine::engine::AudioEngine;
use std::thread;
use std::time::Duration;
use std::io::{Write};


fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("--- Audio Engine Example ---");

    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");
    let path = r"D:\Downloads\Rehta hoon khud mein gum sa aksar.mp3";
    engine.load(path)?;
    engine.play()?;
    engine.set_bass_boost(true);
    println!("Playback started. Press Enter to stop...");

    // Main loop to keep the engine ticking and handle device changes
    let mut input = String::new();
    let mut prev_time = -1.0;
    let mut current_time = -2.0;
    loop {
        engine.tick();
        if prev_time != current_time {
            prev_time = current_time;
            current_time = engine.get_time_secs();
        }

        if current_time == prev_time {
            break;
        }

        println!("\rPlayback time: {:.2} seconds", current_time);
        thread::sleep(Duration::from_secs(1));
    }

    println!("Engine example finished.");
    Ok(())
}
