
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//
// Search
//
#[derive(Debug, Clone, Serialize, Default)]
pub struct SearchRequestBuilder {
    pub query: Option<String>,
    pub channel_id: Option<String>,
}

impl SearchRequestBuilder {
    pub(crate) fn build<S: Into<String>>(self, api_key: S) -> SearchRequest {
        SearchRequest {
            part: String::from("snippet"),
            key: api_key.into(),
            query: self.query,
            _type: Some(String::from("video")),
            max_results: Some(1),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    part: String,
    key: String,
    #[serde(rename = "q")]
    query: Option<String>,
    #[serde(rename = "type")]
    _type: Option<String>,
    max_results: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
    pub region_code: Option<String>,
    pub page_info: PageInfo,
    pub items: Vec<T>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: u64,
    pub results_per_page: u64,
}

pub type SearchResponse = Response<SearchResult>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub kind: String,
    pub etag: String,
    pub id: Id,
    pub snippet: SearchResultSnippet,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultSnippet {
    pub published_at: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: HashMap<String, Thumbnail>,
    pub channel_title: String,
    pub live_broadcast_content: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Id {
    #[serde(rename = "youtube#video")]
    VideoId {
        #[serde(rename = "videoId")]
        video_id: String
    },
    #[serde(rename = "youtube#channel")]
    ChannelId {
        #[serde(rename = "channelId")]
        channel_id: String
    },
    #[serde(rename = "youtube#playlist")]
    PlaylistId {
        #[serde(rename = "playlistId")]
        playlist_id: String
    },
}

impl Id {
    pub fn into_inner(self) -> String {
        match self {
            Id::VideoId { video_id } => video_id,
            Id::ChannelId { channel_id } => channel_id,
            Id::PlaylistId { playlist_id } => playlist_id,
        }
    }
}

//
// Playlist
//
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSnippet {
    pub published_at: Option<String>,
    pub channel_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tumbnails: Option<HashMap<String, Thumbnail>>,
    pub channel_title: Option<String>,
    pub localized: Option<PlaylistLocalization>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistStatus {
    /// The playlist's privacy status.
    pub privacy_status: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub snippet: PlaylistSnippet,
    pub status: PlaylistStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistLocalization {
    /// The localized strings for playlist's description.
    pub description: Option<String>,
    /// The localized strings for playlist's title.
    pub title: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistResponse {
    // pub kind: String,
    //pub etag: String,
    pub id: String,
    // pub snippet: Option<PlaylistSnippet>,
    // pub status: Option<PlaylistStatus>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistContentDetails {
    /// The number of videos in the playlist.
    pub item_count: Option<u32>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistPlayer {
    /// An <iframe> tag that embeds a player that will play the playlist.
    pub embed_html: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    snippet: PlaylistItemSnippet,
}

impl PlaylistItem {
    pub fn new(playlist_id:String, video_id:String) -> PlaylistItem {
        PlaylistItem { 
            snippet: PlaylistItemSnippet {
                playlist_id: playlist_id,
                resource_id: PlaylistItemResource {
                    kind: "youtube#video".to_string(),
                    video_id: video_id,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemSnippet {
    playlist_id: String,
    resource_id: PlaylistItemResource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemResource {
    kind: String,
    video_id: String,
}

// Errors
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleErrorResponse {
    pub error: GoogleError,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleError {
    code: u16,
    errors: Vec<ErrorItem>,
    message: String,
}
impl std::fmt::Display for GoogleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for GoogleError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorItem {
    domain: String,
    message: String,
    reason: String,
}