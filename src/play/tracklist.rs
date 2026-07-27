// Subsonic album and track metadata.


use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct SubsIDResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: ResponseBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResponseBody {
    pub status: String,
    pub version: String,
    #[serde(rename = "type")]
    pub server_type: String,
    #[serde(rename = "serverVersion")]
    pub server_version: String,
    #[serde(rename = "openSubsonic")]
    pub open_subsonic: bool,
    pub album: Album,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Album {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub artist: String,
    #[serde(rename = "artistId", default)]
    pub artist_id: String,
    #[serde(rename = "coverArt", default)]
    pub cover_art: String,
    #[serde(rename = "songCount", default)]
    pub song_count: u32,
    #[serde(default)]
    pub duration: u32,
    #[serde(rename = "playCount", default)]
    pub play_count: u32,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub year: u32,
    #[serde(default)]
    pub played: Option<String>,
    #[serde(rename = "userRating", default)]
    pub user_rating: u32,
    #[serde(default)]
    pub genres: Vec<serde_json::Value>,
    #[serde(rename = "musicBrainzId", default)]
    pub music_brainz_id: String,
    #[serde(rename = "isCompilation", default)]
    pub is_compilation: bool,
    #[serde(rename = "sortName", default)]
    pub sort_name: String,
    #[serde(default)]
    pub artists: Vec<ArtistRef>,
    #[serde(rename = "displayArtist", default)]
    pub display_artist: String,

    #[serde(default)]
    pub song: Vec<Song>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ArtistRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ReplayGain {
    #[serde(rename = "trackGain")]
    pub track_gain: f32,
    #[serde(rename = "albumGain")]
    pub album_gain: f32,
    #[serde(rename = "trackPeak")]
    pub track_peak: f32,
    #[serde(rename = "albumPeak")]
    pub album_peak: f32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Song {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub parent: String,
    #[serde(rename = "isDir", default)]
    pub is_dir: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub track: u32,
    #[serde(default)]
    pub year: u32,
    #[serde(rename = "coverArt", default)]
    pub cover_art: String,
    #[serde(default)]
    pub size: u64,
    #[serde(rename = "contentType", default)]
    pub content_type: String,
    #[serde(default)]
    pub suffix: String,
    #[serde(default)]
    pub duration: u32,
    #[serde(rename = "bitRate", default)]
    pub bit_rate: u32,
    #[serde(default)]
    pub path: String,
    #[serde(rename = "playCount", default)]
    pub play_count: Option<u32>,
    #[serde(default)]
    pub created: String,
    #[serde(rename = "artistId", default)]
    pub artist_id: String,
    #[serde(rename = "type", default)]
    pub media_type: String,
    #[serde(default)]
    pub played: Option<String>,
    #[serde(default)]
    pub bpm: u32,
    #[serde(default)]
    pub comment: String,
    #[serde(rename = "sortName", default)]
    pub sort_name: String,
    #[serde(rename = "mediaType", default)]
    pub media_type_tag: String,
    #[serde(rename = "channelCount", default)]
    pub channel_count: u32,
    #[serde(rename = "samplingRate", default)]
    pub sampling_rate: u32,
    #[serde(rename = "bitDepth", default)]
    pub bit_depth: u32,
    #[serde(rename = "displayArtist", default)]
    pub display_artist: String,
    #[serde(rename = "displayAlbumArtist", default)]
    pub display_album_artist: String,
}


pub struct MpdAlbum<'a> {
    file: &'a str,
    title: &'a str,
    artist: &'a str,
    album: &'a str,
    duration: f32,
    track: i16,
}


/* Fetch an album's MPD tracklist data. */
impl SubsIDResponse {
    pub async fn from_id(client: &Client, album_id: &str) -> SubsIDResponse {
        let url = format!("http://192.168.1.20:8097/rest/getAlbum?id={}&u=nix&p=2008&v=1.8.0&c=myapp&f=json", album_id);
        client
            .get(url)
            .send().await.unwrap()
            .error_for_status().unwrap()
            .json::<SubsIDResponse>()
            .await.unwrap()
    }

    pub async fn new(client: &Client, uid: &str, _ser: &str) -> SubsIDResponse {
        let url = format!("http://192.168.1.20:8097/rest/getAlbum?id={}&u=nix&p=2008&v=1.8.0&c=myapp&f=json", uid);
        let root = client
            .get(url)
            .query(&[
                ("f", "json"),
                ("type", "alphabeticalByName"),
                ("size", "500"),
                ("offset", "0"),
            ])
            .send()
            .await.unwrap()
            .error_for_status().unwrap()
            .json::<SubsIDResponse>()
        .await.unwrap();
        root
    }
    fn get_tracklist(&self) -> Vec<&str> {
        let mut vec: Vec<&str> = Vec::new();
        let album_list: Vec<Song> = self
            .subsonic_response
            .album
            .song.clone();
        println!("array of song found (dbg");
        for _i in album_list.iter() {
            let mpdretrn: &str  = 
                "file: {} \n
                Last-Modified: {} \n
                Time: {} \n
                duration: {} \n
                Artist: {} \n
                AlbumArtist: {} \n
                Title: {} \n
                Track: {} \n
                Date: {} \n
                Genre: {} \n
                ";
            vec.push(mpdretrn);


        }
        vec

    }


}
