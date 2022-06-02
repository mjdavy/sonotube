
use sonos::{self, Track};
use std::{collections::HashSet};
use youtube_api::{YoutubeApi};
use std::io;
use youtube_api::models::SearchRequestBuilder;

pub struct Tube {
    pub seen: HashSet<String>,
    api:Option<YoutubeApi>,
}

pub fn sonotube_login(url: String) -> String {
    
    println!("1. Open this URL in your browser:\n\n{}\n", url);
    println!("2. Copy access code from browser\n");
    println!("3. Paste access code below:");

    let mut code = String::new();
    io::stdin().read_line(&mut code).unwrap();

    code
}

impl Tube {
    pub fn new() -> Tube {
        Tube { 
            seen: HashSet::new(),
            api: None
        }
    }

    pub async fn authenticate(&mut self) -> bool {
        use std::env;

        let client_id: String = match env::var("CLIENT_ID") {
            Ok(id) => id,
            Err(e) => {
                 eprintln!("CLIENT_ID {e}"); 
                 return false;
            },
        };
    
        let client_secret: String = match env::var("CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(e) => {
                eprintln!("CLIENT_SECRET {e}");
                return false;
            }
        };

        let api_key: String = match env::var("API_KEY") {
            Ok(secret) => secret,
            Err(e) => {
                eprintln!("API_KEY {e}");
                return false;
            }
        };

        let api = match YoutubeApi::new_with_oauth (
            api_key, client_id, client_secret,None)
            {
                Ok(api) => api,
                Err(err) => {
                    eprintln!("Youtube authorization failed {}", err.to_string());
                    return false;
                },
            };

        match api.load_token().await {
            Ok(()) => {
                eprintln!("Loaded token");
            }
            Err(msg) => {
                eprintln!("Unable to load token {:?}", msg);
            }
        }

        if !api.has_token() {
        
            api.login(sonotube_login).await.unwrap();  
            let result = api.store_token().await;
            match result {
                Ok(()) => (),
                Err(msg) => {
                    eprintln!("Unable to store token: {:?}", msg);
                }
            }
        }

        self.api = Some(api);  
        println!("\nAuthenticated successfully\n");
        return true;
    }

    pub async fn process_track(&mut self, track: &Track) {

        eprintln!("Tube:: Received {} by {}", track.title, track.artist);
        if self.seen.insert(track.uri.clone()) {
            eprintln!("Tube::processing track {} by {}", track.title, track.artist);
            self.find_track_info(track).await;
        }
        else {
            eprintln!("Tube::ingoring track {} by {} - already processed", track.title, track.artist);
        }
    }

    async fn find_track_info(&mut self, track: &Track) {
        let request = SearchRequestBuilder {
           query: Some(format!("{} - {}", track.title, track.artist)),
           channel_id: None
        };
        
        match self.api.as_ref().unwrap().search(request).await {
            Ok(val) => {
                eprintln!("Search returned: {:?}", val);
            }
            Err(msg) => {
                eprintln!("Search failed: {}", msg);
            }
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
    let success = tube.authenticate().await;
    assert_eq!(true, success);
    tube.process_track(&track).await;
}

