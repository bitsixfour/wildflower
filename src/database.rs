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
pub struct FindArgs {
    pub filter: String,
    pub sort: Option<String>,
    pub window_start: Option<u32>,
    pub window_end: Option<u32>,
    pub position: Option<u32>,
}
pub enum DatabaseStatus {
    AlbumArt(String, i64),
    Count(String, String),
    Find(FindArgs),
    FindAdd(FindArgs),
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

fn song_sort_key(song: &Song, field: &str) -> String {
    match field.to_lowercase() {
        "title" | "name" => song.title.to_lowercase(),
        "artist" => song.artist.to_lowercase(),
        "album" => song.album.to_lowercase(),
        "artistsort" => {
            if !song.sort_name.is_empty() {
                song.sort_name.to_lowercase()
            } else {
                song.artist.to_lowercase()
            }
        }
        "albumartist" | "albumartistsort" => {
            if !song.display_album_artist.is_empty() {
                song.display_album_artist.to_lowercase()
            } else if !song.display_artist.is_empty() {
                song.display_artist.to_lowercase()
            } else {
                song.artist.to_lowercase()
            }
        }
        "albumsort" => {
            if !song.sort_name.is_empty() {
                song.sort_name.to_lowercase()
            } else {
                song.album.to_lowercase()
            }
        }
        "track" => format!("{:04}", song.track),
        "year" | "date" => song.year.to_string(),
        "duration" => format!("{:08}", song.duration),
        "last-modified" => song.created.clone(),
        "id" => song.id.clone(),
        _ => String::new(),
    }
}

fn format_song(song: &Song) -> String {
    let mut out = String::new();
    out.push_str(&format!("file: {}\n", song.path));
    out.push_str(&format!("Title: {}\n", song.title));
    out.push_str(&format!("Artist: {}\n", song.artist));
    out.push_str(&format!("Album: {}\n", song.album));
    out.push_str(&format!("Track: {}\n", song.track));
    out.push_str(&format!("Year: {}\n", song.year));
    out.push_str(&format!("Duration: {}\n", song.duration));
    out.push_str(&format!("Id: {}\n", song.id));
    if !song.artist_id.is_empty() {
        out.push_str(&format!("ArtistId: {}\n", song.artist_id));
    }
    out
}

async fn collect_songs(
    expr: &Expr,
    client: &Client,
    navi: &NaviData,
) -> Vec<Song> {
    let mut matches = Vec::new();
    for album in &navi.album_list {
        let songs = get_album_songs(client, navi, album).await;
        for song in &songs {
            if expr.matches_song(song) {
                matches.push(song.clone());
            }
        }
    }
    matches
}

fn apply_sort(matches: &mut Vec<Song>, sort: &Option<String>) {
    let Some(sort_field) = sort else { return };
    let desc = sort_field.starts_with('-');
    let field = sort_field.trim_start_matches('-');
    if field.is_empty() {
        return;
    }
    matches.sort_by(|a, b| {
        let ka = song_sort_key(a, field);
        let kb = song_sort_key(b, field);
        if desc { kb.cmp(&ka) } else { ka.cmp(&kb) }
    });
}

fn apply_window(matches: &mut Vec<Song>, start: Option<u32>, end: Option<u32>) {
    let s = start.unwrap_or(0) as usize;
    let e = end.unwrap_or(matches.len() as u32) as usize;
    if s >= matches.len() {
        matches.clear();
    } else {
        let e = e.min(matches.len());
        *matches = matches[s..e].to_vec();
    }
}

pub async fn database_handle(command: DatabaseStatus, client: &Client, navi: NaviData) -> String {
    match command {
        DatabaseStatus::AlbumArt(id, ost) => {
            art::return_album_art(id, ost).await
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
        DatabaseStatus::Find(args) => {
            let expr = match parser::parse_filter(&args.filter) {
                Some(e) => e,
                _ => return "ACK [2@0] {find} could not parse filter\n".to_string(),
            };

            let mut matches = collect_songs(&expr, client, &navi).await;
            apply_sort(&mut matches, &args.sort);
            apply_window(&mut matches, args.window_start, args.window_end);

            let mut out = String::new();
            for song in &matches {
                out.push_str(&format_song(song));
                out.push('\n');
            }
            out.push_str("OK\n");
            out
        }
        DatabaseStatus::FindAdd(args) => {
            let expr = match parser::parse_filter(&args.filter) {
                Some(e) => e,
                _ => return "ACK [2@0] {findadd} could not parse filter\n".to_string(),
            };

            let mut matches = collect_songs(&expr, client, &navi).await;
            apply_sort(&mut matches, &args.sort);
            apply_window(&mut matches, args.window_start, args.window_end);

            let mut out = String::new();
            for song in &matches {
                out.push_str(&format_song(song));
                out.push('\n');
            }
            out.push_str("OK\n");
            out
        }
        _ => format!("ACK-!"),
    }
}
