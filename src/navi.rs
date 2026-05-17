use std::collections::HashMap;
use serde::Deserialize;
use reqwest::Client;

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "subsonic-response")]
    subsonic_response: SubsonicResponse,
}

#[derive(Debug, Deserialize)]
pub struct SubsonicResponse {
    status: String,
    version: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "serverVersion")]
    server_version: String,
    #[serde(rename = "openSubsonic")]
    open_subsonic: bool,
    #[serde(rename = "albumList2")]
    album_list_2: AlbumList2,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
struct AlbumList2 {
    album: Vec<Album>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
struct Album {
    id: String,
    name: String,
    artist: String,

    #[serde(rename = "artistId")]
    artist_id: String,

    #[serde(rename = "coverArt")]
    cover_art: String,

    #[serde(rename = "songCount")]
    song_count: u32,

    duration: u32,

    created: String,

    year: Option<u32>,
    genre: Option<String>,

    #[serde(rename = "userRating")]
    user_rating: Option<u32>,

    genres: Vec<Genre>,

    #[serde(rename = "musicBrainzId")]
    music_brainz_id: Option<String>,

    #[serde(rename = "isCompilation")]
    is_compilation: bool,

    #[serde(rename = "sortName")]
    sort_name: Option<String>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
struct Genre {
    name: String,
}

pub async fn navi_obj(client: &Client) -> Result<SubsonicResponse, reqwest::Error> {
    let root = client
        .get("http://127.0.0.1:8097/rest/getAlbumList2.view")
        .query(&[
            ("f", "json"),
            ("type", "alphabeticalByName"),
            ("size", "500"),
            ("offset", "0"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<Root>()
        .await?;

    Ok(root.subsonic_response)
}


pub struct NaviData<'a> {
    pub data: HashMap<&'a str, &'a Album>
}
impl NaviData<'a> {

    pub async fn new(client: &Client) -> NaviData {
        let mut hmap: HashMap<Album, String> = HashMap::new();
        let resp: SubsonicResponse = navi_obj(client).await.expect("Could not access Navidrome. Check the URL and Navidrome itself");
        let album: Vec<Album> = resp.album_list_2.album;
        
        for i in &album {
            println!("adding to hashmap (dbg)");
            let x: &str = i.name.as_str();
            hmap.insert(i, x);
 
        }
        Ok(Self {
            data: hmap,
        })
    }



    pub async fn get_url(song_id: &str) -> String {
a       
    }
}
