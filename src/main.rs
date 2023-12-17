use async_std::task;
use chrono;
use dirs;
use models::TubeTrack;
use serde::{Deserialize, Serialize};
use sonos::{self, Track};
use tube::Tube;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, fs::OpenOptions};
use tokio::task::JoinHandle;
use env_logger::Env;

const TRACK_CACHE: &str = ".sonotube_tracks.json";
const CONFIG: &str = ".sonotube.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Config {
    api_key: Option<String>,
    create_playlist: Option<bool>,
    send_previous_tracks: Option<bool>,
    create_toptastic_playlist: Option<bool>,
}

impl From<Track> for TubeTrack {
    fn from(track: Track) -> Self {
        TubeTrack {
            id: track.uri,
            title: track.title,
            artist: track.artist,
        }
    }
}

impl Config {
    fn new() -> Self {
        let config = match Config::load(CONFIG) {
            Some(config) => {
                if let Some(value) = &config.api_key {
                    std::env::set_var(tube::API_KEY_VAR, value)
                }
                config
            }
            None => Config {
                api_key: None,
                create_playlist: None,
                send_previous_tracks: None,
                create_toptastic_playlist: None,
            },
        };
        config
    }

    pub fn create_play_list(&self) -> bool {
        match self.create_playlist {
            Some(val) => val,
            None => false,
        }
    }

    pub fn send_previous_tracks(&self) -> bool {
        match self.send_previous_tracks {
            Some(val) => val,
            None => false,
        }
    }

    fn load(file_name: &str) -> Option<Self> {
        use std::fs;
        let config_path = get_config_path(file_name);

        if !config_path.exists() {
            return None;
        }

        let serialized = fs::read_to_string(config_path).expect("Unable to load config");
        serde_json::from_str(&serialized).unwrap()
    }

    pub fn _save(&self, file_name: &str) {
        let config_path = get_config_path(file_name);

        let log = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&config_path)
            .expect(format!("Unable to open {:?} for writing", &config_path).as_str());

        let mut serializer = serde_json::Serializer::new(log);
        self.serialize(&mut serializer)
            .expect("Unable to save config");
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    play_history: Option<Vec<i64>>,
}

impl Clone for SerTrack {
    fn clone(&self) -> Self {
        Self {
            track: Track {
                title: self.track.title.clone(),
                artist: self.track.artist.clone(),
                album: self.track.album.clone(),
                queue_position: self.track.queue_position,
                uri: self.track.uri.clone(),
                duration: self.track.duration.clone(),
                running_time: self.track.running_time.clone(),
            },
            play_history: self.play_history.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.track = source.clone().track;
        self.play_history = source.play_history.clone();
    }
}

mod models;
mod tube;
mod toptastic;

#[tokio::main]
async fn main() {

    let config = Config::new();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let (sender, receiver) = mpsc::channel::<Track>();
    let track_monitor_flag = Arc::new(AtomicBool::new(true));

    println!("Hit enter to quit");
    wait_for_enter_key(track_monitor_flag.clone());

    let track_monitor_handle =
        start_track_monitor(sender, track_monitor_flag.clone(), config).await;
    let tube_monitor_handle = start_tube_monitor(receiver).await;
    
    start_toptastic_server().await.expect("toptastic server failed");

    track_monitor_handle.await.expect("track_monitor panicked");
    tube_monitor_handle.await.expect("tube_monitor panicked");
   

    println!("Done.");
}

fn wait_for_enter_key(flag: Arc<AtomicBool>) {
    tokio::spawn(async move {
        match io::stdin().read_line(&mut String::new()) {
            Ok(_) => {
                println!("Shuting down... Please wait");
                flag.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Err(_) => eprintln!("Error reading stdin"),
        }
    });
}

async fn start_toptastic_server() -> std::io::Result<()> {
    println!("Starting toptastic server...");

    let toptastic = toptastic::TopTastic::new().await.unwrap();
    toptastic.start_server().await
    
}

async fn start_tube_monitor(receiver: mpsc::Receiver<Track>) -> JoinHandle<()> {
    println!("Starting tube monitor...");
    tokio::spawn(async move {
        let mut tube = tube::Tube::new();
        for track in receiver {
            let tube_track = TubeTrack::from(track);
            let (title, description) = Tube::generate_sonotube_title_and_description("sonotube");
            tube.process_track(&tube_track, &title, &description).await;
        }
    })
}

async fn start_track_monitor(
    sender: mpsc::Sender<Track>,
    flag: Arc<AtomicBool>,
    config: Config,
) -> JoinHandle<()> {
    println!("Starting track monitor...");
    tokio::spawn(async move {
        // find sonos devices on the network
        let devices = sonos::discover().await.unwrap();
        println!("Found {} devices on your network", devices.len());

        let mut tracks = load_tracks(TRACK_CACHE);

        if config.send_previous_tracks() {
            for ser_track in tracks.values() {
                let track = ser_track.clone().track;
                sender.send(track).unwrap();
            }
        }

        // poll found devices for new tracks
        let mut last_track_uri = String::from("");
        while flag.load(std::sync::atomic::Ordering::Relaxed) {
            for device in &devices {
                if let Ok(track) = device.track().await {
                    // Check if the track has changed
                    if &track.uri == &last_track_uri {
                        continue;
                    }
                    let title = track.title.clone();
                    let artist = track.artist.clone();
                    last_track_uri = track.uri.clone();

                    // See if we played this track before
                    if tracks.contains_key(&track.uri) {
                        let ser_track = tracks.get_mut(&track.uri).unwrap();
                        match ser_track.play_history {
                            Some(ref mut history) => {
                                history.push(chrono::Utc::now().timestamp());
                            }
                            None => {
                                let mut history = Vec::new();
                                history.push(chrono::Utc::now().timestamp());
                                ser_track.play_history = Some(history);
                            }
                        }
                    } else {
                        let now = chrono::Utc::now();
                        let ser_track = SerTrack {
                            track: track,
                            play_history: Some(vec![now.timestamp()]),
                        };

                        // add this track to the youtube playlist if config option is enabled
                        if config.create_play_list() {
                            sender.send(ser_track.clone().track).unwrap();
                        }

                        tracks.insert(ser_track.track.uri.clone(), ser_track);
                    }
                    println!("{} by {} is playing on {}", title, artist, device.name);
                }
            }

            save_tracks(TRACK_CACHE, &tracks);
            task::sleep(Duration::from_secs(30)).await
        }
        println!("Track monitor exiting...")
    })
}

fn load_tracks(file_name: &str) -> HashMap<String, SerTrack> {
    use std::fs;
    let tracks_path = get_tracks_path(file_name);

    if !tracks_path.exists() {
        return HashMap::new();
    }

    let serialized = fs::read_to_string(tracks_path).expect("Unable to load tracks");
    serde_json::from_str(&serialized).unwrap()
}

fn save_tracks(file_name: &str, tracks: &HashMap<String, SerTrack>) {
    let tracks_path = get_tracks_path(file_name);

    let log = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&tracks_path)
        .expect(format!("Unable to open {:?} for writing", &tracks_path).as_str());

    let mut serializer = serde_json::Serializer::new(log);
    tracks
        .serialize(&mut serializer)
        .expect("Unable to save tracks");
}

fn get_config_path(file_name: &str) -> PathBuf {
    let mut config_path = dirs::home_dir().expect("The home directory was not found.");
    config_path.push(file_name);
    config_path
}

fn get_tracks_path(file_name: &str) -> PathBuf {
    let mut tracks_path = dirs::cache_dir().expect("The cache directory was not found.");
    tracks_path.push(file_name);
    tracks_path
}

#[tokio::test]
async fn test_load_save_tracks() {
    let track = Track {
        title: "test_title".to_string(),
        artist: "test_artist".to_string(),
        album: Some("test_album".to_string()),
        queue_position: 1,
        uri: "test_uri".to_string(),
        duration: Duration::from_secs(10),
        running_time: Duration::from_secs(100),
    };

    let mut track_map = HashMap::new();
    track_map.insert(
        track.uri.clone(),
        SerTrack {
            track: track,
            play_history: Some(vec![0]),
        },
    );

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

#[tokio::test]
async fn test_config() {
    let config = Config::new();
    config._save(".test_sonotube_config.json");
}
