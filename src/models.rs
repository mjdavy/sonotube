
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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchListResponse {
    /// Etag of this resource.
    pub etag: Option<String>,
    /// Serialized EventId of the request which produced this response.
    pub event_id: Option<String>,
    /// Pagination information for token pagination.
    pub items: Option<Vec<SearchResult>>,
    /// Identifies what kind of resource this is. Value: the fixed string "youtube#searchListResponse".
    pub kind: Option<String>,
    /// The token that can be used as the value of the pageToken parameter to retrieve the next page in the result set.
    pub next_page_token: Option<String>,
    /// General pagination information.
    pub page_info: Option<PageInfo>,
    /// The token that can be used as the value of the pageToken parameter to retrieve the previous page in the result set.
    pub prev_page_token: Option<String>,
    /// no description provided
    pub region_code: Option<String>,
    /// no description provided
    pub token_pagination: Option<TokenPagination>,
    /// The visitorId identifies the visitor.
    pub visitor_id: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPagination {
    /// Tokens to pass to the standard list field 'page_token'. Whenever available, tokens are preferred over manipulating start_index.
    pub next_page_token: Option<String>,
    /// no description provided
    pub previous_page_token: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// Etag of this resource.
    pub etag: Option<String>,
    /// The id object contains information that can be used to uniquely identify the resource that matches the search request.
    pub id: Option<ResourceId>,
    /// Identifies what kind of resource this is. Value: the fixed string "youtube#searchResult".
    pub kind: Option<String>,
    /// The snippet object contains basic details about a search result, such as its title or description. For example, if the search result is a video, then the title will be the video's title and the description will be the video's description.
    pub snippet: Option<SearchResultSnippet>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ResourceId {
    /// Required field for the type-specific id. This should correspond to the id used in the type-specific API's.
    pub id: Option<String>,
    /// Required field representing the resource type this id is for. At present, the valid types are "project", "folder", and "organization".
    #[serde(rename="type")]
    pub type_: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    /// Maximum number of results returned in one page. ! The number of results included in the API response.
    pub result_per_page: Option<i32>,
    /// Index of the first result returned in the current page.
    pub start_index: Option<i32>,
    /// Total number of results available on the backend ! The total number of results in the result set.
    pub total_results: Option<i32>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultSnippet {
    /// The value that YouTube uses to uniquely identify the channel that published the resource that the search result identifies.
    pub channel_id: Option<String>,
    /// The title of the channel that published the resource that the search result identifies.
    pub channel_title: Option<String>,
    /// A description of the search result.
    pub description: Option<String>,
    /// It indicates if the resource (video or channel) has upcoming/active live broadcast content. Or it's "none" if there is not any upcoming/active live broadcasts.
    pub live_broadcast_content: Option<String>,
    /// The creation date and time of the resource that the search result identifies.
    pub published_at: Option<String>,
    /// A map of thumbnail images associated with the search result. For each object in the map, the key is the name of the thumbnail image, and the value is an object that contains other information about the thumbnail.
    pub thumbnails: Option<ThumbnailDetails>,
    /// The title of the search result.
    pub title: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    /// (Optional) Height of the thumbnail image.
    pub height: Option<u32>,
    /// The thumbnail image's URL.
    pub url: Option<String>,
    /// (Optional) Width of the thumbnail image.
    pub width: Option<u32>,
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
