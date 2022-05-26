

use std::collections::HashSet;
use sonos::{self, Track};
use sonos::Speaker;
use async_std::{task};
use std::time::Duration;
use async_std::prelude::*;
use async_std::fs::OpenOptions;
use duration_string::DurationString;
use std::sync::mpsc;
use std::thread;
use std::env::{self, VarError};

mod tube;

static TRACK_FILE_PATH: &str = "tracks.log";
const TRACK_REGEX: &str = "Track \\{ title: \"(.+?)\", artist: \"(.+?)\", album: (Some(.+?)|None), queue_position: (.+?), uri: \"(.+?)\", duration: (.+?), running_time: (.+?) \\}";


#[tokio::main]
async fn main()  {
    
    let client_id: String = match env::var("CLIENT_ID") {
        Ok(id) => id,
        Err(e) => {
             eprintln!("CLIENT_ID {e}"); 
             return;
        },
    };

    let client_secret: String = match env::var("CLIENT_SECRET") {
        Ok(secret) => secret,
        Err(e) => {
            eprintln!("CLIENT_SECRET {e}");
            return;
        }
    };

    // find sonos devices on the network and print them out
    println!("Looking for sonos devices...");

    let devices = match sonos::discover().await {
        Ok(speakers) => speakers, 
        Err(err) => { 
            eprintln!("{}", err); 
            return; 
        },
    };

    println!("Found {} devices", devices.len());

    let sender = start_tube_monitor(client_id, client_secret);
    monitor_tracks(&devices,TRACK_FILE_PATH, &sender).await;

}

async fn load_tracks(path: &str, tracks: &mut Vec<Track>) 
{
    use async_std::io::BufReader;
    use async_std::fs::File;

    eprintln!("Loading tracks file...");

    let open_file_for_read = File::open(path).await;

    let file = match open_file_for_read {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Unable to open track file: {:?}", err);
            return;
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
        if let Some(track) = parse_track(line) {
            tracks.push(track);
        }
    }
}

#[test]
fn test_load_tracks()
{
    let path = "tracks_load_test.log";
    let mut tracks = Vec::new();
    task::block_on(load_tracks(path, &mut tracks)); 
    assert_eq!(40,tracks.len());
}

fn parse_track(line: &str) -> Option<Track>
{
    use regex::Regex;
    let re = Regex::new(TRACK_REGEX).unwrap();
    let cap = re.captures(line).unwrap();
    assert_eq!(cap.len(),9);

    let album = |n: String| { if n == "None" {None} else { Some(cap[4].to_string()) } };
    
    let track: Track = Track {
        title: cap[1].to_string(), 
        artist: cap[2].to_string(), 
        album: album(cap[3].to_string()), 
        queue_position: u64::from_str_radix(&cap[5], 10).unwrap(), 
        uri: cap[6].to_string(),
        duration: DurationString::from_string(cap[7].to_string()).unwrap().into(), 
        running_time: DurationString::from_string(cap[8].to_string()).unwrap().into() 
    };

    Some(track)
}

#[test]
fn test_duration_string()
{
    let nanos_20:Duration = DurationString::from_string("20ns".to_string()).unwrap().into();
    assert_eq!(nanos_20.subsec_nanos(),20);
}

#[test]
fn test_parse_track_some() {
    let line = "Track { title: \"Interstellar\", artist: \"Deep Forest & Gaudi\", album: Some(\"Epic Circuits\"), queue_position: 1, uri: \"x-sonos-http:librarytrack%3ai.qYglBfAaQNa0.mp4?sid=204&flags=8232&sn=3\", duration: 290s, running_time: 40s }";
    let track: Track = Track {
        title: "Interstellar".to_string(), 
        artist: "Deep Forest & Gaudi".to_string(), 
        album: Some("(\"Epic Circuits\")".to_string()), 
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

#[test]
fn test_parse_track_none() {
    let line = "Track { title: \"title\", artist: \"artist\", album: None, queue_position: 2, uri: \"uri\", duration: 300s, running_time: 50s }";
    let track: Track = Track {
        title: "title".to_string(), 
        artist: "artist".to_string(), 
        album: None, 
        queue_position: 2, 
        uri: "uri".to_string(),
        duration: Duration::from_secs(300),
        running_time: Duration::from_secs(50)
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

fn start_tube_monitor(client_id: String, client_secret: String) -> mpsc::Sender<Track>
{
    // Build the communication channel
   
    let (sender, receiver) = mpsc::channel::<Track>();
    thread::spawn(|| {
        for track in receiver {
            let mut tube = tube::Tube::new();
            tube.process_track(&track);
        }
    });
    return sender;
}

async fn monitor_tracks(devices: &Vec<Speaker>, path:&str, sender:&mpsc::Sender<Track>)
{
   
    // load previously seen track uris, so we don't get duplicates
    let mut tracks = Vec::new();
    load_tracks(path, &mut tracks).await; 

    // extract uris of previously seen tracks - send them to tube
    let mut seen_tracks = HashSet::new();
    for track in tracks {
        seen_tracks.insert(track.uri.clone());
        let track_string: String = format!("{:?}", track);
        if sender.send(track).is_err() {
           eprintln!("Send failed for track {}", track_string);
        }
    }

    loop {
        for device in devices {
            if let Ok(track) = device.track().await {
                let res = process_track(&track, path, &mut seen_tracks).await;
                if let Err(err) = res {
                    eprintln!("{:?}", err);
                } 
                
                if sender.send(track).is_err() {
                    eprintln!("Send failed");
                }
                
            } 
        }
        task::sleep(Duration::from_secs(30)).await
    }
    
}

async fn process_track(track: &Track, path: &str, tracks: &mut HashSet<String>) -> Result<(),std::io::Error>
{
    if tracks.contains(&track.uri) {
        return Ok(());
    }

    tracks.insert(track.uri.clone());

    // log the track
    let mut buffer = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .await?;

    buffer.write_fmt(format_args!("{:?}\n", track)).await?;

    Ok(())
}

#[test]
fn test_process_track() {
    let test_track_path = "tracks_test.log";

    let track: Track = Track {
        title: "title".to_string(), 
        artist: "artist".to_string(), 
        album: Some("album".to_string()), 
        queue_position: 1, 
        uri: "uri".to_string(),
        duration: Duration::from_secs(180), 
        running_time: Duration::from_secs(10) 
    };

    let mut tracks = HashSet::new();
    let result = task::block_on(process_track(&track, test_track_path, &mut tracks));
    assert!(tracks.contains(&track.uri));
    assert_eq!((), result.unwrap());
}

