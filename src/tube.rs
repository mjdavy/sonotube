
use dirs;
use sonos::{self, Track};
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use crate::models::*;
use std::env;
    
const CLIENT_SECRETS_PATH: &str = r"D:\secrets\sonotube\client_secrets.json";
const TOKEN_CACHE_FILE: &str = "sonotube_token_cache.json";
const PLAYLISTS_URI: &str = "https://www.googleapis.com/youtube/v3/playlists";
const SEARCH_URI: &str = "https://www.googleapis.com/youtube/v3/search";

pub struct Tube {
    pub seen: HashSet<String>,
    token: Option<AccessToken>,
}

impl Tube {
    pub fn new() -> Tube {
        Tube {
            seen: HashSet::new(),
            token: None,
        }
    }

    fn get_token_cache_path(&mut self, file_name: &str) -> PathBuf {
        let mut token_cache = dirs::cache_dir().expect("The cache directory was not found.");
        token_cache.push(file_name);
        token_cache
    }

    async fn authenticate(&mut self)  {
        // Load the client secrets from the client_secrets.json path.
        let secrets_path = Path::new(CLIENT_SECRETS_PATH);
        let secret = yup_oauth2::read_application_secret(secrets_path)
            .await
            .expect(secrets_path.to_str().unwrap());

        // Create an authenticator that uses an InstalledFlow to authenticate. The
        // authentication tokens are persisted to a file named tokencache.json. The
        // authenticator takes care of caching tokens to disk and refreshing tokens once
        // they've expired.
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(self.get_token_cache_path(TOKEN_CACHE_FILE).to_str().unwrap())
                .build()
                .await
                .unwrap();

        // Obtain a token that can be sent e.g. as Bearer token.
        let scopes = &["https://www.googleapis.com/auth/youtube"];

        self.token = match auth.token(scopes).await {
            Ok(token) => Some(token),
            Err(err) => {
                eprintln!("Failed to obtain access token: {:?}", err);
                panic!("{:?}",err);
            }
        }
    }

    pub async fn process_track(&mut self, track: &Track) {
        eprintln!("Tube:: Received {} by {}", track.title, track.artist);
        if self.seen.insert(track.uri.clone()) {
            eprintln!("Tube::processing track {} by {}", track.title, track.artist);
            if let Some(video_id) = self.find_video_id_for_track(track).await {
                eprintln!("Now get video info for {}", video_id);
            }
        } else {
            eprintln!(
                "Tube::ingoring track {} by {} - already processed",
                track.title, track.artist
            );
        }
    }

    async fn insert_playlist(&mut self, playlist_title: &str, playlist_description: &str) {
        
        if self.token.is_none() {
            self.authenticate().await;
        }

        let playlist = Playlist { 
            snippet: PlaylistSnippet {
                title: Some(String::from(playlist_title)), 
                description: Some(String::from(playlist_description)),
                channel_id: None,
                channel_title: None,
                default_language: None,
                localized: None,
                published_at: None,
                tags: None,
                thumbnail_video_id: None,
                thumbnails: None,
            },
            status: PlaylistStatus {
                privacy_status: Some(String::from("private")),
            },
        };

        let token_str = self.token.as_ref().unwrap().as_str();
    
        let client = reqwest::Client::new();
        let res = client.post(PLAYLISTS_URI)
            .query(&[("part","snippet,status")])
            .bearer_auth(token_str)
            .json(&playlist)
            .send()
            .await;
        match res {
            Ok(res) => {
                println!("");
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    async fn find_video_id_for_track(&mut self, track: &Track) -> Option<String> {

        if self.token.is_none() {
            self.authenticate().await;
        }
    
        let request = SearchRequestBuilder {
            query: Some(format!("{} - {}", track.title, track.artist)),
            channel_id: None,
        };

        let api_key: String = match env::var("SONOTUBE_API_KEY") {
            Ok(secret) => secret,
            Err(e) => {
                println!("SONOTUBE_API_KEY {e}");
                return None;
            }
        };

        let token_str = self.token.as_ref().unwrap().as_str();
        let client = reqwest::Client::new();
        let res = client.get(SEARCH_URI)
            .query(&request.build(api_key.as_str()))
            .bearer_auth(token_str)
            .send()
            .await;
        match res {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => println!("Error: {}", e),
        }
        return None;
    }
}

#[tokio::test]
async fn test_process_track() {
    use std::time::Duration;

    let track = Track {
        title: String::from("title"),
        artist: String::from("artist"),
        album: Some(String::from("album")),
        queue_position: 1,
        uri: String::from("uri"),
        duration: Duration::from_secs(180),
        running_time: Duration::from_secs(1),
    };

    let mut tube = Tube::new();
    tube.process_track(&track).await;
}
