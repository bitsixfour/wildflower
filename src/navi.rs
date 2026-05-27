use std::collections::HashMap;
use serde::Deserialize;
use reqwest::Client;

const URL: &str = "http://192.168.1.20:8097";
const USR: &str = "nix";

#[derive(Debug, Deserialize, Clone)]
pub struct Root {
    #[serde(rename = "subsonic-response")]
    subsonic_response: SubsonicResponse,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Eq, Hash, Clone,  PartialEq)]
pub struct AlbumList2 {
    album: Vec<Album>,
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq)]
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

    pub created: String,

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

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct Genre {
    name: String,
}
pub fn get_url(song_id: &str) -> String {
    format!("{}/rest/stream?id={}&u={USR}&v=1.8.0&c=myapp", 
        URL, song_id)
}

pub async fn navi_obj(client: &Client) -> Result<SubsonicResponse, reqwest::Error> {
    let root = client
        .get("http://192.168.1.20:8097/rest/getAlbumList2?u=nix&p=2008&v=1.16.1&c=test&f=json&type=alphabeticalByName&size=500") /* YEAH YOU HAVE MY PASSWORD */
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
pub struct NaviData {
    pub data: HashMap<String, Album>, /* test CLI utils */
    pub album_list: Vec<Album>,
}

impl NaviData {
    pub async fn new(resp: SubsonicResponse) -> Self {
        let mut hmap: HashMap<String, Album> = HashMap::new();
        println!("Mapping Navidrome...");
        let album: Vec<Album> = resp.album_list_2.album;
        for i in &album {
            let name = i.name
                .clone()
                .to_lowercase();
            println!("Yes saar. We are importing this to navidrome");
            hmap.insert(name, i.clone()); 
            album.push(i.clone());
        }
        Self {
            data: hmap,
            album_list: album,
        }
    }
}
