use std::collections::HashMap;

use crate::navi::{Album, NaviData};
use crate::parser;
use crate::search::Expr;
use crate::tracklist::{self, Song};

pub enum QueueStatus {
    Add(String, i32),
    AddId(String, i32),
    Clear(), 
    Delete(String),
    DeleteId(String),
    Move(String),
    MoveId(String, String),
    Playlistfind(String, String),
    PlaylistId(String),
    PlaylistInfo(String),
    PlaylistSearch(String),
    PiChanges(String, (i32, i32)),
    PiChangesPos(String, (i32, i32)),
    Prio(i32, (i32, i32)),
    PrioId(i32, (i32, i32))
}
pub enum DatabaseStatus {
    AlbumArt(String, i64),
    Count(String, String),
    Find(String, String),
    FindAdd(Vec<&str>),
    Lis(Vec<&str>),
    ListAll(Box<&str>),
    ListAllInfo(Box<&str>),
    ListFiles(&str),
    LsInfo(&str),
    ReadComment(&str),
    ReadPicture(&str),
    SearchAdd(Vec<&str>),
    Searchaddpi(Vec<&str>),
    SearchCount(Vec<&str>),
    Update(),
    Rescan()
}

async fn get_album_songs(client: &Client, navi: &NaviData, album: &Album) -> Vec<Song> {
    {
        let cache = navi.songs_cache.read().unwrap();
        if let Some(songs) = cache.get(&album.id) {
            return songs.clone();
        }
    }
    if let Ok(resp) = client
        .get(&format!(
            "http://192.168.1.20:8097/rest/getAlbum?id={}&u=nix&p=2008&v=1.8.0&c=myapp&f=json",
            album.id
        ))
        .send()
        .await
    {
        if let Ok(parsed) = resp.json::<tracklist::SubsIDResponse>().await {
            let songs = parsed.subsonic_response.album.song;
            let mut cache = navi.songs_cache.write().unwrap();
            cache.insert(album.id.clone(), songs.clone());
            return songs;
        }
    }
    Vec::new()
}

fn song_group_value(song: &Song, group_type: &str) -> String {
    match group_type.to_lowercase().as_str() {
        "title" => song.title.clone(),
        "artist" => song.artist.clone(),
        "album" => song.album.clone(),
        "year" | "date" => song.year.to_string(),
        "track" => song.track.to_string(),
        "genre" => String::new(),
        _ => String::new(),
    }
}

pub async fn database_handle(command: DatabaseStatus, client: &Client, navi: NaviData) -> String {
    match command {
        DatabaseStatus::AlbumArt(id, ost) => {
            let resp = art::return_album_art(id, ost).await;
            resp
        }
        DatabaseStatus::Count(filter_str, group_type) => {
            let expr = match parser::parse_filter(&filter_str) {
                Some(e) => e,
                None => return "ACK [2@0] {count} could not parse filter\n".to_string(),
            };

            let mut total_songs: u32 = 0;
            let mut total_playtime: u32 = 0;
            let mut groups: HashMap<String, (u32, u32)> = HashMap::new();
            let has_group = !group_type.is_empty();

            for album in &navi.album_list {
                let songs = get_album_songs(client, &navi, album).await;
                for song in &songs {
                    if expr.matches_song(song) {
                        total_songs += 1;
                        total_playtime += song.duration;
                        if has_group {
                            let key = song_group_value(song, &group_type);
                            let entry = groups.entry(key).or_insert((0, 0));
                            entry.0 += 1;
                            entry.1 += song.duration;
                        }
                    }
                }
            }

            let mut out = format!("songs: {}\nplaytime: {}\n", total_songs, total_playtime);
            if has_group {
                let mut group_vec: Vec<(String, (u32, u32))> = groups.into_iter().collect();
                group_vec.sort_by(|a, b| a.0.cmp(&b.0));
                for (key, (sc, pt)) in &group_vec {
                    out.push_str(&format!("songs: {}\nplaytime: {}\ngroup: {}\n", sc, pt, key));
                }
            }
            out.push_str("OK\n");
            out
        }
        DatabaseStatus::Find(_, _) => {
            "OK\n".to_string()
        }
        _ => format!("ACK-!"),
    }
}
