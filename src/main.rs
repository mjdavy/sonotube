use std::collections::HashSet;
use sonos::{self, Track};
use async_std::{task};
use tokio::task::JoinHandle;
use std::time::Duration;
use async_std::prelude::*;
use duration_string::DurationString;
use std::sync::mpsc;


mod tube;

const TRACK_REGEX: &str = "Track \\{ title: \"(.+?)\", artist: \"(.+?)\", album: (Some(.+?)|None), queue_position: (.+?), uri: \"(.+?)\", duration: (.+?), running_time: (.+?) \\}";

#[tokio::main]
async fn main()  {

    let (sender, receiver) = mpsc::channel::<Track>();

    let track_monitor_handle = start_track_monitor(sender).await;
    let tube_monitor_handle = start_tube_monitor(receiver).await;
    
    track_monitor_handle.await.expect("track_monitor panicked");
    tube_monitor_handle.await.expect("tube_monitor panicked");
}

async fn start_tube_monitor(receiver:mpsc::Receiver<Track>) -> JoinHandle<()>
{
    tokio::spawn(async move {
        let mut tube = tube::Tube::new();
        if !tube.authenticate().await {
            panic!("YouTube authentication failed");
        }
        
        for track in receiver {
            tube.process_track(&track).await;
        }
    })
}

async fn start_track_monitor(sender:mpsc::Sender<Track>) -> JoinHandle<()>
{
    tokio::spawn(async move  {

        let track_file_path = "tracks.log".to_string();
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
            
        // load previously seen track uris, so we don't get duplicates
        let mut tracks = Vec::new();
        load_tracks(&track_file_path, &mut tracks).await; 

        // extract uris of previously seen tracks - send them to tube
        let mut seen_tracks = HashSet::new();
        for track in tracks {
            seen_tracks.insert(track.uri.clone());
            let track_string: String = format!("{} - {}", track.title,track.artist);
            match sender.send(track) {
                Ok(()) => (),
                Err(msg) => eprintln!("Send failed for track {} - {:?}", track_string, msg.to_string()),
            }
        }
   
        loop {
            for device in &devices {
                if let Ok(track) = device.track().await {
                    seen_tracks.insert(track.uri.clone());
                    
                    if sender.send(track).is_err() {
                        eprintln!("Send failed");
                    }
                } 
            }
            task::sleep(Duration::from_secs(30)).await
        }
    })
    
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
fn test_load_tracks()
{
    let path = "tracks_load_test.log";
    let mut tracks = Vec::new();
    task::block_on(load_tracks(path, &mut tracks)); 
    assert_eq!(40,tracks.len());
}

#[test]
fn test_duration_string()
{
    let nanos_20:Duration = DurationString::from_string("20ns".to_string()).unwrap().into();
    assert_eq!(nanos_20.subsec_nanos(),20);
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


