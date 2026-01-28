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
    engine.load(path)?;
    println!("Audio file loaded: {}", path);
    engine.play()?;
    println!("Playback started.");
    thread::sleep(Duration::from_secs(120));


    println!("Engine example finished.");
    Ok(())
}
