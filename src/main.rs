use sonos::{self, Track};
use async_std::{task};
use tokio::task::JoinHandle;
use std::time::Duration;
use async_std::prelude::*;
use duration_string::DurationString;
use std::sync::mpsc;
use serde::{Serialize, Deserialize};
use std::{fs::OpenOptions, io::Write, collections::HashMap};
use std::path::PathBuf;
use dirs;

const TRACK_CACHE: &str = ".sonotube_tracks.json";

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "Track")]
struct TrackDef {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub queue_position: u64,
    pub uri: String,
    pub duration: Duration,
    pub running_time: Duration,
}

#[derive(Serialize, Deserialize, Debug)]
struct SerTrack {
    #[serde(with = "TrackDef")]
    track: Track,
}

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
        // find sonos devices on the network
        println!("Looking for sonos devices...");
        let devices = match sonos::discover().await {
            Ok(speakers) => speakers, 
            Err(err) => { 
                eprintln!("{}", err); 
                return; 
            },
        };
        println!("Found {} devices", devices.len());
        
        // poll found devices for new tracks
        loop {
            for device in &devices {
                if let Ok(track) = device.track().await {
                    if sender.send(track).is_err() {
                        eprintln!("Send failed");
                    }
                } 
            }
            task::sleep(Duration::from_secs(30)).await
        }
    })
    
}

fn load_tracks(file_name: &str) -> HashMap<String, SerTrack>
{
    use std::fs;
    let tracks_path = get_tracks_path(file_name);

    if !tracks_path.exists() {
        return HashMap::new();
    }

    let serialized = fs::read_to_string(tracks_path).expect("Unable to load tracks");
    serde_json::from_str(&serialized).unwrap()
}

fn save_tracks(file_name: &str, tracks: &HashMap<String, SerTrack>)
{
    let tracks_path = get_tracks_path(file_name);

    let mut log = OpenOptions::new()
    .write(true)
    .create(true)
    .open(&tracks_path).expect(format!("Unable to open {:?} for writing", &tracks_path).as_str());

    let serialized = serde_json::to_string(&tracks).unwrap();
    log.write_all(serialized.as_bytes()).expect("Unable to save tracks");
}

fn get_tracks_path(file_name: &str) -> PathBuf
{
    let mut tracks_path = dirs::home_dir().expect("The home directory was not found.");
    tracks_path.push(file_name);
    tracks_path
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
fn test_load_save_tracks()
{
    let track = Track {
        title: "test_title".to_string(), 
        artist: "test_artist".to_string(), 
        album: Some("test_album".to_string()), 
        queue_position: 1, 
        uri: "test_uri".to_string(),
        duration: Duration::from_secs(10),
        running_time: Duration::from_secs(100)
    };

    let mut track_map = HashMap::new();
    track_map.insert(track.uri.clone(), SerTrack {track: track});

    let test_file_name = ".test_track_cache.json";
    save_tracks(test_file_name, &track_map);
    assert!(get_tracks_path(test_file_name).exists());

    let loaded_tracks = load_tracks(test_file_name);
    assert_eq!(1, loaded_tracks.len());

    let test_track = loaded_tracks.values().take(1).next().unwrap();

    assert_eq!("test_title", &test_track.track.title);
    assert_eq!("test_artist", &test_track.track.artist);
    assert_eq!("test_album", test_track.track.album.as_ref().unwrap());
    assert_eq!(1, test_track.track.queue_position);
    assert_eq!("test_uri", &test_track.track.uri);
    assert_eq!(Duration::from_secs(10), test_track.track.duration);
    assert_eq!("test_artist", &test_track.track.artist);
    
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


