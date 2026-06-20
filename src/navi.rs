use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Context, Result};


use crate::tracklist::SubsIDResponse;
use crate::tracklist::Song;
// main way to get metadata and parse actual library. TODO: it's 500 max albums but for a POC (for
// now) it's good enough
const URL: &str = "http://192.168.1.20:8097";
const USR: &str = "nix";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Root {
    #[serde(rename = "subsonic-response")]
    subsonic_response: SubsonicResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Eq, Hash, Clone, PartialEq)]
pub struct AlbumList2 {
    album: Vec<Album>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
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

    pub year: Option<u32>,
    pub genre: Option<String>,

    #[serde(rename = "userRating")]
    pub user_rating: Option<u32>,

    pub genres: Vec<Genre>,

    #[serde(rename = "musicBrainzId")]
    pub music_brainz_id: Option<String>,

    #[serde(rename = "isCompilation")]
    pub is_compilation: bool,

    #[serde(rename = "sortName")]
    pub sort_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct Genre {
    name: String,
}
pub fn get_url(song_id: &str) -> String {
    format!("{}/rest/stream?id={}&u={USR}&v=1.8.0&c=myapp", URL, song_id)
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



#[derive(Clone)]
pub struct NaviData {
    pub data: HashMap<String, Album>, 
    pub data_id: HashMap<String, Album>, 
    pub album_list: Vec<Album>,
    pub songs_cache: HashMap<String, Vec<Song>>,
    pub albums_cache: HashMap<String, Vec<Song>>
}
// turn into hashmap
impl NaviData {
    pub fn init_empty() -> Self {
        Self {
            data: HashMap::new(),
            data_id: HashMap::new(),
            album_list: Vec::new(),
            songs_cache: HashMap::new(),
            albums_cache: HashMap::new(),
        }
    }
    // the "key" in the hash is album.name
    pub async fn init_cache(data: Vec<Album>, client: &Client) -> HashMap<String, Vec<Song>>  {
        let mut h_map: HashMap<String, Vec<Song>> = HashMap::new();
        for idx in &data {
            let key: (&str, &str) = (&idx.id, &idx.name);
            let metadata: Vec<Song> =  SubsIDResponse::from_id(client, key.0).await
                .subsonic_response
                .album
                .song;
            h_map.insert(key.0.to_string(), metadata);
        } h_map

    }
    pub async fn updt(resp: SubsonicResponse, clnt: &Client) -> Self {
        let mut hmap: HashMap<String, Album> = HashMap::new();
        let mut hmap_2: HashMap<String, Album> = HashMap::new();
        let album: Vec<Album> = resp.album_list_2.album;
        for i in &album {
            let name = i.name.clone().to_lowercase();
            let id = i.id.clone().to_lowercase();
            hmap.insert(name, i.clone());
            hmap_2.insert(id,i.clone());
        }
        // take ownership, not needed anymore
        let kv_cache: HashMap<String, Vec<Song>> = Self::init_cache(album.clone(), clnt).await;
        Self {
            data: hmap,
            data_id: hmap_2,
            album_list: album,
            songs_cache: HashMap::new(),
            albums_cache: kv_cache,
        }
    }

    /// Build a NaviData by reusing the on-disk cache and only fetching the
    /// per-album song lists for albums we haven't seen before. The album-list
    /// endpoint (`getAlbumList2`) is always called once because it's cheap
    /// and lets us detect new or removed albums.
    pub async fn load_or_fetch(client: &Client, cache_path: &Path) -> Result<Self> {
        let mut cache = Cache::load(cache_path).unwrap_or_default();

        // 1. Cheap: always hit the album list
        let fresh = navi_obj(client).await.context("getAlbumList2 failed")?;
        let fresh_albums = fresh.album_list_2.album;

        // 2. Decide which albums we still need to fetch
        let cached_ids: HashSet<String> = cache.album_songs.keys().cloned().collect();
        let missing: Vec<Album> = fresh_albums
            .iter()
            .filter(|a| !cached_ids.contains(&a.id))
            .cloned()
            .collect();
        let fetched_n = missing.len();

        // 3. Per-album song fetch — only for new albums
        if !missing.is_empty() {
            let new_songs = Self::init_cache(missing, client).await;
            for (id, songs) in new_songs {
                cache.album_songs.insert(
                    id,
                    AlbumSongs {
                        fetched_at: unix_now(),
                        songs,
                    },
                );
            }
        }

        // 4. Build the in-memory NaviData
        let mut data: HashMap<String, Album> = HashMap::new();
        let mut data_id: HashMap<String, Album> = HashMap::new();
 for album in &fresh_albums {
            data.insert(album.name.to_lowercase(), album.clone());
            data_id.insert(album.id.to_lowercase(), album.clone());
        }
        let mut albums_cache: HashMap<String, Vec<Song>> = HashMap::new();
        for (id, bucket) in &cache.album_songs {
            albums_cache.insert(id.clone(), bucket.songs.clone());
        }

        // 5. Persist (atomic) — skip the write when nothing changed
        if fetched_n > 0 {
            cache.album_list_raw = fresh_albums.clone();
            cache.fetched_at = unix_now();
            if let Err(e) = cache.save(cache_path) {
                eprintln!("warning: could not write cache to {}: {e}", cache_path.display());
            }
        }

        Ok(NaviData {
            data,
            data_id,
 album_list: fresh_albums,
            songs_cache: HashMap::new(),
            albums_cache,
        })
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    /// Bump when the on-disk format changes incompatibly.
    #[serde(default)]
    version: u32,
    #[serde(default)]
    fetched_at: u64,
    /// Raw album list as returned by `getAlbumList2`.
    #[serde(default)]
    album_list_raw: Vec<Album>,
    /// Per-album song buckets, keyed by album id (lowercased).
    #[serde(default)]
    album_songs: HashMap<String, AlbumSongs>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlbumSongs {
    #[serde(default)]
    fetched_at: u64,
    songs: Vec<Song>,
}

impl Cache {
    fn load(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        match serde_json::from_slice::<Self>(&bytes) {
            Ok(c) if c.version <= 1 => Some(c),
            Ok(_) => {
                eprintln!("cache {} has newer version, ignoring", path.display());
                None
            }
            Err(e) => {
                eprintln!("cache {} is corrupt ({e}), ignoring", path.display());
                None
            }
        }
    }

    fn save(&self, path: &Path) -> std::io::Result<()> {
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)
    }
}
