use crate::models::*;
use dirs;
use reqwest::Client;
use sonos::{self, Track};
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const CLIENT_SECRETS_PATH: &str = r"D:\secrets\sonotube\client_secrets.json";
const TOKEN_CACHE_FILE: &str = "sonotube_token_cache.json";
const PLAYLISTS_URI: &str = "https://www.googleapis.com/youtube/v3/playlists";
const SEARCH_URI: &str = "https://www.googleapis.com/youtube/v3/search";
const PLAYLIST_ITEMS_URI: &str = "https://www.googleapis.com/youtube/v3/playlistItems";

pub struct Tube {
    pub seen: HashSet<String>,
    token: Option<AccessToken>,
    client: Client,
    playlist_id: Option<String>,
}

impl Tube {
    pub fn new() -> Tube {
        Tube {
            seen: HashSet::new(),
            token: None,
            client: Client::new(),
            playlist_id: None,
        }
    }

    fn get_token_cache_path(&mut self, file_name: &str) -> PathBuf {
       
        let mut token_cache = dirs::cache_dir().expect("The cache directory was not found.");
        token_cache.push(file_name);
        token_cache
    }

    async fn authenticate(&mut self) {
        
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
                .persist_tokens_to_disk(
                    self.get_token_cache_path(TOKEN_CACHE_FILE)
                        .to_str()
                        .unwrap(),
                )
                .build()
                .await
                .unwrap();

        // Obtain a token that can be sent e.g. as Bearer token.
        let scopes = &["https://www.googleapis.com/auth/youtube"];

        self.token = match auth.token(scopes).await {
            Ok(token) => Some(token),
            Err(err) => {
                eprintln!("Failed to obtain access token: {:?}", err);
                panic!("{:?}", err);
            }
        }
    }

    pub async fn process_track(&mut self, track: &Track) {
        
        if self.playlist_id.is_none() {
            let now = chrono::Local::now().format("%a %b %e %Y %T").to_string();
            let title = format!("sonotube - {}", now);
            let description = format!("playlist created by sonotube on {}", now);
            self.playlist_id = self.insert_playlist(&title, &description).await;
        }

        eprintln!("Tube:: Received {} by {}", track.title, track.artist);
        if self.seen.insert(track.uri.clone()) {
            eprintln!("Tube::processing track {} by {}", track.title, track.artist);
            match self.find_video_id_for_track(track).await {
                Some(video_id) => self.add_video_to_playlist(&self.playlist_id.clone().unwrap(), &video_id).await,
                None => eprintln!("Tube:: No video found for {} by {}", track.title, track.artist),
            }
        } else {
            eprintln!(
                "Tube::ingoring track {} by {} - already processed",
                track.title, track.artist
            );
        }
    }

    async fn insert_playlist(
        &mut self,
        playlist_title: &str,
        playlist_description: &str,
    ) -> Option<String> {
       
        if self.token.is_none() {
            self.authenticate().await;
        }

        let mut playlist = Playlist::default();
        playlist.snippet.title = Some(String::from(playlist_title));
        playlist.snippet.description = Some(String::from(playlist_description));
        playlist.status.privacy_status = Some(String::from("private"));

        let token_str = self.token.as_ref().unwrap().as_str();

        let result = self
            .client
            .post(PLAYLISTS_URI)
            .query(&[("part", "snippet,status")])
            .bearer_auth(token_str)
            .json(&playlist)
            .send()
            .await;

        let response = match result {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error: failed to instert playlist. {:?}", err);
                return None;
            }
        };

        if response.error_for_status_ref().is_ok() {
            let playlist_result: Result<PlaylistResponse, reqwest::Error> = response.json().await;
            match playlist_result {
                Ok(playlist_result) => {
                    println!("{:?}", playlist_result);
                    return Some(playlist_result.id);
                }
                Err(e) => {
                    eprintln!("Error: failed to parse playlist response: {:?}", e);
                    return None;
                }
            }
        } else {
            let err: GoogleErrorResponse = response.json().await.unwrap();
            eprintln!("{:?}", err);
            return None;
        }
    }

    async fn find_video_id_for_track(&mut self, track: &Track) -> Option<String> {
        let search_request = SearchRequestBuilder {
            query: Some(format!("{} {}", track.title, track.artist)),
            channel_id: None,
        };

        let api_key: String = match env::var("SONOTUBE_API_KEY") {
            Ok(secret) => secret,
            Err(e) => {
                println!("SONOTUBE_API_KEY {e}");
                return None;
            }
        };

        let request = search_request.build(api_key);
        let result = self.client.get(SEARCH_URI).query(&request).send().await;

        let response = match result {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error: failed to get search results. {:?}", err);
                return None;
            }
        };

        if response.error_for_status_ref().is_ok() {
            let search_result: Result<SearchResponse, reqwest::Error> = response.json().await;
            match search_result {
                Ok(search_result) => {
                    let items = search_result.items;
                    let id = items.first().map(|item| &item.id).unwrap();
                    let video_id = &id.clone().into_inner();
                    return Some(video_id.into());
                }
                Err(e) => {
                    eprintln!("Error: failed to parse search results: {:?}", e);
                    None
                }
            }
        } else {
            let err: GoogleErrorResponse = response.json().await.unwrap();
            eprintln!("{:?}", err);
            return None;
        }
    }

    async fn add_video_to_playlist(&mut self, playlist_id: &str, video_id: &str) {
        if self.token.is_none() {
            self.authenticate().await;
        }

        let playlist_video = PlaylistItem::new(String::from(playlist_id), String::from(video_id));
        let token_str = self.token.as_ref().unwrap().as_str();

        let res = self
            .client
            .post(PLAYLIST_ITEMS_URI)
            .query(&[("part", "snippet")])
            .bearer_auth(token_str)
            .json(&playlist_video)
            .send()
            .await;
        match res {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => println!("Error: {}", e),
        }
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

#[tokio::test]
async fn test_find_video_id_for_track() {
    use std::time::Duration;

    let track = Track {
        title: String::from("shape of you"),
        artist: String::from("ed shiran"),
        album: None,
        queue_position: 1,
        uri: String::from("uri"),
        duration: Duration::from_secs(0),
        running_time: Duration::from_secs(0),
    };

    let mut tube = Tube::new();
    let res = tube.find_video_id_for_track(&track).await;
    println!("{:?}", res);
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "JGwWNGJdvx8");
}

#[tokio::test]
async fn test_add_video_to_playlist() {
    let mut tube = Tube::new();
    tube.add_video_to_playlist("PLtZ7tJkCfjGxIK-bH7fodXCmpEDmEvebL", "JGwWNGJdvx8")
        .await;
}

#[tokio::test]
async fn test_insert_playlist() {
    let mut tube = Tube::new();
    let id = tube.insert_playlist("test", "test").await;
    assert!(id.is_some());
}
