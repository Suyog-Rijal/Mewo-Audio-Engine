mod engine;

use std::{io, thread};
use std::io::Write;
use std::time::Duration;
use crate::engine::engine::AudioEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Audio Engine Example ---");
    let mut engine = AudioEngine::new()?;
    println!("Engine initialized.");

    let path = r"D:\Downloads\test.mp3";
    engine.load(path)?;
    // if let Some(meta) = engine.get_metadata() {
    //     println!("Loaded File Info:");
    //     println!("  Title:   {}", meta.title.as_deref().unwrap_or("Unknown"));
    //     println!("  Artist:  {}", meta.artist.as_deref().unwrap_or("Unknown"));
    //     println!("  Album:   {}", meta.album.as_deref().unwrap_or("Unknown"));
    //
    //     if let Some(dur) = meta.duration_secs {
    //         println!("Duration: {}", dur as u64);
    //     } else {
    //         println!("  Length:  Unknown");
    //     }
    // } else {
    //     println!("No metadata found.");
    // }

    println!("Playback started...");
    engine.play()?;
    let mut counter = 0;
    loop {
        if counter == 10 {
            break;
        }
        thread::sleep(Duration::from_secs(1));
        counter += 1;
    }

    engine.load_and_play(r"D:\Downloads\test 2.mp3")?;
    let mut seconds_played = 0;
    while engine.is_playing() && seconds_played < 10 {
        print!("\rPlaying Song 2 - Time: {:.2}s", engine.get_time_secs());
        io::stdout().flush()?;

        thread::sleep(Duration::from_secs(1));
        seconds_played += 1;
    }

    println!("Playback finished.");
    Ok(())
}