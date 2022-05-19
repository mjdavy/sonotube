

use std::collections::HashSet;
use sonos::{self, Track};
use sonos::Speaker;
use async_std::task;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // find sonos devices on the network and print them out
    println!("Looking for sonos devices...");

    let devices = sonos::discover().await?;
    println!("Found {} devices", devices.len());
    
    monitor_tracks(&devices).await;
    Ok(())
}

async fn play(speaker: &Speaker) {
    let result = speaker.play().await;
    if let Ok(()) = result {
        eprintln!("Track is playing")
    } else {
        eprintln!("Something went wrong")
    }
    ()
}

async fn find_media_arc(devices: &Vec<Speaker>) -> &Speaker{
     devices.iter()
        .find(|d| d.model == "Sonos Arc")
        .expect("Unable to find any sonos Arc on your network")
}

async fn monitor_tracks(devices: &Vec<Speaker>)
{
    let mut tracks = HashSet::new();
    loop {
        for device in devices {
            if let Ok(track) = device.track().await {
                let res = process_track(&track, &mut tracks).await;
                if let Err(err) = res {
                    eprint!("{:?}", err)
                } 
            } else {
                eprintln!("No current track on {} - {}", device.name, device.ip)
            }
        }
        tracks.insert(String::from("foo"));
        task::sleep(Duration::from_secs(30)).await
    }
}

async fn process_track(track: &Track, tracks: &mut HashSet<String>) -> Result<(),std::io::Error>
{
    use async_std::prelude::*;
    use async_std::fs::OpenOptions;

    if tracks.contains(&track.uri) {
        return Ok(());
    }

    tracks.insert(track.uri.clone());
    println!("New Track: {:?}", track);

    // log the track
    let logfile = String::from("tracks.log");
    let mut buffer = OpenOptions::new()
            .append(true)
            .create(true)
            .open(logfile)
            .await?;

    buffer.write_fmt(format_args!("{:?}\n", track)).await?;

    Ok(())
}

