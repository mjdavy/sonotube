
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
            channel_id: self.channel_id,
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
    channel_id: Option<String>,
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
    /// The ID that YouTube uses to uniquely identify the channel that published the playlist.
    pub channel_id: Option<String>,
    /// The channel title of the channel that the video belongs to.
    pub channel_title: Option<String>,
    /// The language of the playlist's default title and description.
    pub default_language: Option<String>,
    /// The playlist's description.
    pub description: Option<String>,
    /// Localized title and description, read-only.
    pub localized: Option<PlaylistLocalization>,
    /// The date and time that the playlist was created.
    pub published_at: Option<String>,
    /// Keyword tags associated with the playlist.
    pub tags: Option<Vec<String>>,
    /// Note: if the playlist has a custom thumbnail, this field will not be populated. The video id selected by the user that will be used as the thumbnail of this playlist. This field defaults to the first publicly viewable video in the playlist, if: 1. The user has never selected a video to be the thumbnail of the playlist. 2. The user selects a video to be the thumbnail, and then removes that video from the playlist. 3. The user selects a non-owned video to be the thumbnail, but that video becomes private, or gets deleted.
    pub thumbnail_video_id: Option<String>,
    /// A map of thumbnail images associated with the playlist. For each object in the map, the key is the name of the thumbnail image, and the value is an object that contains other information about the thumbnail.
    pub thumbnails: Option<ThumbnailDetails>,
    /// The playlist's title.
    pub title: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDetails {
    /// The default image for this resource.
    pub default: Option<Thumbnail>,
    /// The high quality image for this resource.
    pub high: Option<Thumbnail>,
    /// The maximum resolution quality image for this resource.
    pub maxres: Option<Thumbnail>,
    /// The medium quality image for this resource.
    pub medium: Option<Thumbnail>,
    /// The standard quality image for this resource.
    pub standard: Option<Thumbnail>,
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
pub struct PlaylistResource {
    /// Identifies what kind of resource this is. Value: the fixed string "youtube#playlist".
    pub kind: Option<String>,
    /// Etag of this resource.
    pub etag: Option<String>,
    /// The ID that YouTube uses to uniquely identify the playlist.
    pub id: Option<String>,
    /// The snippet object contains basic details about the playlist, such as its title and description.
    pub snippet: Option<PlaylistSnippet>,
    /// The status object contains status information for the playlist.
    pub status: Option<PlaylistStatus>,
    /// The contentDetails object contains information like video count.
    pub content_details: Option<PlaylistContentDetails>,
    /// The player object contains information that you would use to play the playlist in an embedded player.
    pub player: Option<PlaylistPlayer>,
    /// Localizations for different languages
    pub localizations: Option<HashMap<String, PlaylistLocalization>>,
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