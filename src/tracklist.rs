use std::collections::HashMap;
use reqwest::Client;
use serde::Deserialize;
use crate::NaviData;


#[derive(Debug, Deserialize)]
pub struct SubsIDResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: ResponseBody,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: String,
    #[serde(rename = "artistId")]
    pub artist_id: String,
    #[serde(rename = "coverArt")]
    pub cover_art: String,
    #[serde(rename = "songCount")]
    pub song_count: u32,
    pub duration: u32,
    #[serde(rename = "playCount")]
    pub play_count: u32,
    pub created: String,
    pub year: u32,
    pub played: Option<String>,
    #[serde(rename = "userRating")]
    pub user_rating: u32,
    pub genres: Vec<serde_json::Value>,
    #[serde(rename = "musicBrainzId")]
    pub music_brainz_id: String,
#[serde(rename = "isCompilation")]
    pub is_compilation: bool,
    #[serde(rename = "sortName")]
    pub sort_name: String,
    pub artists: Vec<ArtistRef>,
    #[serde(rename = "displayArtist")]
    pub display_artist: String,

    /* Song Data.... */
    pub song: Vec<Song>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct Song {
    pub id: String,
    pub parent: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track: u32,
    pub year: u32,
    #[serde(rename = "coverArt")]
    pub cover_art: String,
    pub size: u64,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub suffix: String,
    pub duration: u32,
    #[serde(rename = "bitRate")]
    pub bit_rate: u32,
    pub path: String,
    #[serde(rename = "playCount")]
    pub play_count: Option<u32>,
    pub created: String,
    #[serde(rename = "artistId")]
    pub artist_id: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub played: Option<String>,
    pub bpm: u32,
    pub comment: String,
    #[serde(rename = "sortName")]
    pub sort_name: String,
    #[serde(rename = "mediaType")]
    pub media_type_tag: String,
    #[serde(rename = "channelCount")]
    pub channel_count: u32,
    #[serde(rename = "samplingRate")]
    pub sampling_rate: u32,
    #[serde(rename = "bitDepth")]
    pub bit_depth: u32,
    #[serde(rename = "displayArtist")]
    pub display_artist: String,
    #[serde(rename = "displayAlbumArtist")]
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


/* Actually get MPD Tracklist data from album-id */
impl SubsIDResponse {
    async fn new(client: &Client, alb: &NaviData, ser: &str) -> SubsIDResponse{
        println!("currentsong");
        let uid: &str = alb.data.get(ser).unwrap().id.as_str();
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
        for i in album_list.iter() {
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
