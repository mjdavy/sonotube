use std::sync::Arc;
use crate::{models::TubeTrack, tube::Tube};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Mutex;
use serde::{Deserialize, Serialize};
use actix_web::web::Data;
use log::info;

#[derive(Serialize, Deserialize)]
pub struct Playlist {
    title: String,
    description: String,
    tracks: Vec<TubeTrack>,
}

#[derive(Debug, Clone)]
pub struct TopTastic {
    tube: Tube,
}

impl TopTastic {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tube = Tube::new();
        Ok(Self { tube })
    }

    pub async fn create_playlist(&mut self, title: String, description: String, tracks: Vec<TubeTrack>) -> Vec<TubeTrack> {
        info!("Creating playlist {} with {} tracks", title, tracks.len());
        let mut processed_tracks = Vec::new();
        for track in tracks {
            let video_id = self.tube.process_track(&track, &title, &description).await;
            let processed_track = TubeTrack {
                id: track.id,
                title: track.title,
                artist: track.artist,
                video_id,
            };
            processed_tracks.push(processed_track);
        }
        processed_tracks
    }

    pub async fn start_server(self) -> std::io::Result<()> {
        let port = 3030;
        info!("Starting server on port {}", port);
        let toptastic = TopTastic::new().await.unwrap();
        HttpServer::new(move || {
            App::new()
            .app_data(Data::new(Arc::new(Mutex::new(toptastic.clone()))))
                .service(create_playlist)
                .service(status)
                .service(log_message)
        })
        .bind(("127.0.0.1", port))?
        .run().await
    }
}

#[get("/status")]
async fn status() -> impl Responder {
    info!("Status request received");
    HttpResponse::Ok().body("Server is running")
}

#[post("/log")]
async fn log_message(body: web::Json<String>) -> impl Responder {
    info!("Received message: {}", body.0);
    HttpResponse::Ok().finish()
}

#[post("/playlists")]
async fn create_playlist(data: web::Data<Arc<Mutex<TopTastic>>>, playlist: web::Json<Playlist>) -> impl Responder {
    info!("Create playlist request received");
    let title = playlist.title.clone();
    let description = playlist.description.clone();
    let tracks = playlist.tracks.clone();

    let mut toptastic = data.lock().await;
    let process_tracks = toptastic.create_playlist(title, description, tracks).await;
    HttpResponse::Created().json(process_tracks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn test_create_playlist() {
        let toptastic = TopTastic::new().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(Arc::new(Mutex::new(toptastic))))
                .service(create_playlist)
        ).await;

        let req = test::TestRequest::post()
            .uri("/playlists")
            .set_json(&Playlist {
                title: "Test Playlist".into(),
                description: "Test Description".into(),
                tracks: vec![
                    TubeTrack {
                        id: "test1".into(),
                        title: "we are never getting back together".into(),
                        artist: "Taylor Swift".into(),
                        video_id: None,
                    },
                    TubeTrack {
                        id: "test2".into(),
                        title: "Houdini".into(),
                        artist: "Dua Lipa".into(),
                        video_id: None,
                    }
                ],
            })
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn test_log_message() {
        let toptastic = TopTastic::new().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(Arc::new(Mutex::new(toptastic))))
                .service(log_message)
        ).await;

        let req = test::TestRequest::post()
            .uri("/log")
            .set_json(&"Test message".to_string())
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}