mod engine;

use std::thread;
use crate::engine::engine::AudioEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Audio Engine Example ---");
    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");

    let path = r"D:\Downloads\Tera.mp3";
    engine.load(path)?;
    engine.seek(288.0);
    engine.set_bass_boost(true);

    println!("Playback started...");
    engine.play()?;
    while engine.is_playing() {
        println!("Time: {:} Sec", engine.get_time_secs());
        thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("Playback finished.");
    Ok(())
}