
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::task::JoinHandle;
use env_logger::Env;
use sonos::{self, Track};
use config::Config;
use tube::Tube;
use sonotube::SonoTube;
use models::TubeTrack;

mod models;
mod tube;
mod toptastic;
mod config;
mod sonotube;

impl From<Track> for TubeTrack {
    fn from(track: Track) -> Self {
        TubeTrack {
            id: track.uri,
            title: track.title,
            artist: track.artist,
        }
    }
}

#[tokio::main]
async fn main() {

    let config = Config::new();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let (sender, receiver) = mpsc::channel::<Track>();
    let track_monitor_flag = Arc::new(AtomicBool::new(true));

    println!("Hit enter to quit");
    wait_for_enter_key(track_monitor_flag.clone());

    let track_monitor_handle =
        SonoTube::start_sonos_track_monitor(sender, track_monitor_flag.clone(), config).await;
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
            let (title, description) = Tube::generate_sonotube_title_and_description();
            tube.process_track(&tube_track, &title, &description).await;
        }
    })
}

#[tokio::test]
async fn test_config() {
    let config = Config::new();
    config._save(".test_sonotube_config.json");
}
