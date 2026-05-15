use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NavidromeResponse {
    #[serde(rename = "subsonic-response")]
    subsonic_response: SubsonicResponse,
}

#[derive(Debug, Deserialize)]
struct SubsonicResponse {
    #[serde(rename = "albumList2")]
    album_list2: a,
}

#[derive(Debug, Deserialize)]
struct NowPlaying {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    #[serde(rename = "played")]
    scrobble_time: String,
    title: String,
    album: String,
    artist: String,
}

pub struct Album {
    album: &str,
    artist: &str,
}

pub struct NaviData { /* may change later */
    data: HashMap<Album, String>
}



impl Album {

    pub fn new(str: &String) -> Album {
        
        
    }


}





