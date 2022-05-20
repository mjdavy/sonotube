

use std::collections::HashSet;
use regex::Error;
use sonos::{self, Track};
use sonos::Speaker;
use async_std::{task};
use std::time::Duration;
use async_std::prelude::*;
use async_std::fs::OpenOptions;

const TRACK_FILE_PATH: &str = "tracks.log";
const TRACK_REGEX: &str = "Track \\{ title: \"(.+?)\", artist: \"(.+?)\", album: (.+?), queue_position: (.+?), uri: \"(.+?)\", duration: (.+?), running_time: (.+?) \\}";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // find sonos devices on the network and print them out
    println!("Looking for sonos devices...");

    let devices = sonos::discover().await?;
    println!("Found {} devices", devices.len());
    
    monitor_tracks(&devices).await;
    Ok(())
}

async fn load_tracks(path: &str, tracks: &mut HashSet<String>) 
{
    use async_std::io::BufReader;
    use async_std::fs::File;

    eprintln!("Loading tracks file...");

    let open_file_for_read = File::open(path).await;

    let file = match open_file_for_read {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Unable to open track file: {:?}", err);
            return
        }
    };

    // load buffer
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    if let Err(err) = reader.read_to_string(&mut buffer).await {
        eprintln!("Unable to read track file: {:?}", err); 
        return; 
    }

    // parse contents and build Hashset
    for line in buffer.lines()
    {
        if let Some(track_uri) = parse_track_uri(line) {
            eprintln!("Adding uri: {}", &track_uri);
            tracks.insert(track_uri);
        }
    }
}

fn parse_track_uri(line: &str) -> Option<String>
{
    use regex::Regex;
    let re = Regex::new("uri: \"(.+?)\"").unwrap();
    let cap = re.captures(line).unwrap();
    match cap.get(1) 
    {
        Some(t) => Some(t.as_str().to_string()),
        None => None,
    }    
}

fn parse_track(line: &str) -> Result<Track,Error>
{
    use regex::Regex;
    let re = Regex::new(TRACK_REGEX).unwrap();
    let cap = re.captures(line).unwrap();
    assert_eq!(cap.len(),8);

    let track: Track = Track {
        title: cap[1].to_string(), 
        artist: cap[2].to_string(), 
        album: Some(cap[3].to_string()), 
        queue_position: u64::from_str_radix(&cap[4], 10).unwrap(), 
        uri: cap[5].to_string(),
        duration: Duration::from_secs(290), // TODO
        running_time: Duration::from_secs(40) // TODO
    };

    Ok(track)
}

#[test]
fn test_parse_track() {
    let line = "Track { title: \"Interstellar\", artist: \"Deep Forest & Gaudi\", album: Some(\"Epic Circuits\"), queue_position: 1, uri: \"x-sonos-http:librarytrack%3ai.qYglBfAaQNa0.mp4?sid=204&flags=8232&sn=3\", duration: 290s, running_time: 40s }";
    let track: Track = Track {
        title: "Interstellar".to_string(), 
        artist: "Deep Forest & Gaudi".to_string(), 
        album: Some("Some(\"Epic Circuits\")".to_string()), 
        queue_position: 1, 
        uri: "x-sonos-http:librarytrack%3ai.qYglBfAaQNa0.mp4?sid=204&flags=8232&sn=3".to_string(),
        duration: Duration::from_secs(290),
        running_time: Duration::from_secs(40)
    };
    
    let parsed_track = parse_track(line).unwrap();
    assert_eq!(parsed_track.uri,track.uri);
    assert_eq!(parsed_track.title,track.title);
    assert_eq!(parsed_track.artist,track.artist);
    assert_eq!(parsed_track.album,track.album);
    assert_eq!(parsed_track.queue_position,track.queue_position);
    assert_eq!(parsed_track.uri,track.uri);
    assert_eq!(parsed_track.duration,track.duration);
    assert_eq!(parsed_track.running_time,track.running_time);
}

async fn monitor_tracks(devices: &Vec<Speaker>)
{
    // load previously seen track uris, so we don't get duplicates
    let mut tracks = HashSet::new();
    load_tracks(TRACK_FILE_PATH, &mut tracks).await; 

    loop {
        for device in devices {
            if let Ok(track) = device.track().await {
                let res = process_track(&track, &mut tracks).await;
                if let Err(err) = res {
                    eprint!("{:?}", err)
                } 
            } 
        }
        task::sleep(Duration::from_secs(30)).await
    }
}

async fn process_track(track: &Track, tracks: &mut HashSet<String>) -> Result<(),std::io::Error>
{
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

