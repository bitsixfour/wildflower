use std::collections::{HashMap, HashSet};

use crate::navidrome::navi::{Album, NaviData};
use crate::navidrome::parser;
use crate::navidrome::search::Expr;
use crate::play::art;
use crate::play::tracklist::Song;
use reqwest::Client;

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
    List(ListArgs),
    ListAll(Box<String>),
    ListAllInfo(Box<String>),
    ListFiles(String),
    LsInfo(String),
    ReadComment(String),
    ReadPicture((String, i64)),
    Search(FindArgs),
    SearchAdd(FindArgs),
    Searchaddpi(String, FindArgs),
    SearchCount(ListArgs),
    Update(),
    Rescan(),
}

async fn get_album_songs(navi: &NaviData, album: &Album) -> Vec<Song> {
    navi.albums_cache
        .get(&album.id)
        .cloned()
        .unwrap_or_default()
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

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn song_tag_value(song: &Song, tag: &str) -> String {
    match tag.to_lowercase().as_str() {
        "title" | "name" => song.title.clone(),
        "artist" => song.artist.clone(),
        "album" => song.album.clone(),
        "albumartist" | "album_artist" | "albumartistsort" => {
            if !song.display_album_artist.is_empty() {
                song.display_album_artist.clone()
            } else if !song.display_artist.is_empty() {
                song.display_artist.clone()
            } else {
                song.artist.clone()
            }
        }
        "date" | "year" => song.year.to_string(),
        "track" | "tracknumber" => song.track.to_string(),
        "genre" | "composer" | "disc" | "discnumber" => String::new(),
        "performer" => {
            if !song.display_artist.is_empty() {
                song.display_artist.clone()
            } else {
                song.artist.clone()
            }
        }
        "comment" => song.comment.clone(),
        "filename" | "file" => song.path.clone(),
        "id" => song.id.clone(),
        "duration" => song.duration.to_string(),
        "bitrate" | "bit_rate" => song.bit_rate.to_string(),
        "sortartist" | "artistsort" => {
            if !song.sort_name.is_empty() {
                song.sort_name.clone()
            } else {
                song.artist.clone()
            }
        }
        "albumsort" => {
            if !song.sort_name.is_empty() {
                song.sort_name.clone()
            } else {
                song.album.clone()
            }
        }
        _ => song_group_value(song, tag),
    }
}

fn song_sort_key(song: &Song, field: &str) -> String {
    match field.to_lowercase().as_str() {
        "title" | "name" => song.title.to_lowercase(),
        "artist" | "artistsort" => {
            if !song.sort_name.is_empty() {
                song.sort_name.to_lowercase()
            } else {
                song.artist.to_lowercase()
            }
        }
        "album" => song.album.to_lowercase(),
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
        "last-modified" => song.created.to_string(),
        "id" => song.id.to_string(),
        _ => String::new(),
    }
}

fn format_song(song: &Song) -> String {
    let mut out = format!(
        "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nYear: {}\nDuration: {}\nId: {}",
        song.path,
        song.title,
        song.artist,
        song.album,
        song.track,
        song.year,
        song.duration,
        song.id
    );
    if !song.artist_id.is_empty() {
        out.push_str(&format!("\nArtistId: {}", song.artist_id));
    }
    out.push('\n');
    out
}

async fn collect_songs(expr: &Expr, navi: &NaviData) -> Vec<Song> {
    let mut matches = Vec::new();
    for album in &navi.album_list {
        for song in &get_album_songs(navi, album).await {
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
        let (ka, kb) = (song_sort_key(a, field), song_sort_key(b, field));
        if desc {
            kb.cmp(&ka)
        } else {
            ka.cmp(&kb)
        }
    });
}

fn apply_window(matches: &mut Vec<Song>, start: Option<u32>, end: Option<u32>) {
    let s = start.unwrap_or(0) as usize;
    let e = end
        .map(|value| value as usize)
        .unwrap_or(matches.len())
        .min(matches.len());
    if s >= matches.len() || e <= s {
        matches.clear();
    } else {
        *matches = matches[s..e].to_vec();
    }
}

async fn handle_find(args: FindArgs, kind: &str, navi: &NaviData) -> String {
    let expr = match parser::parse_filter(&args.filter) {
        Some(e) => e,
        None => return format!("ACK [2@0] {{{}}} could not parse filter\n", kind),
    };
    let mut matches = collect_songs(&expr, navi).await;
    apply_sort(&mut matches, &args.sort);
    apply_window(&mut matches, args.window_start, args.window_end);
    let mut out = String::new();
    for song in &matches {
        out.push_str(&format_song(song));
    }
    out.push_str("OK\n");
    out
}
fn field_matches_song_ci(field: &crate::navidrome::search::Field, song: &Song) -> bool {
    let eq = match field.field.to_lowercase().as_str() {
        "title" => song.title.to_lowercase() == field.value.to_lowercase(),
        "artist" => song.artist.to_lowercase() == field.value.to_lowercase(),
        "album" => song.album.to_lowercase() == field.value.to_lowercase(),
        "year" | "date" => song.year.to_string() == field.value,
        "track" => song.track.to_string() == field.value,
        "id" => song.id == field.value,
        _ => false,
    };
    use crate::navidrome::search::FieldOp;
    match field.op {
        FieldOp::Contains => {
            let val = match field.field.to_lowercase().as_str() {
                "title" => &song.title,
                "artist" => &song.artist,
                "album" => &song.album,
                _ => return eq,
            };
            val.to_lowercase().contains(&field.value.to_lowercase())
        }
        FieldOp::Eq => eq,
        FieldOp::NotEq => !eq,
    }
}

fn expr_matches_song_ci(expr: &crate::navidrome::search::Expr, song: &Song) -> bool {
    match expr {
        crate::navidrome::search::Expr::And(a, b) => {
            field_matches_song_ci(a, song) && field_matches_song_ci(b, song)
        }
        crate::navidrome::search::Expr::Or(a, b) => {
            field_matches_song_ci(a, song) || field_matches_song_ci(b, song)
        }
        crate::navidrome::search::Expr::Def(f) => field_matches_song_ci(f, song),
        crate::navidrome::search::Expr::Empty => true,
    }
}

async fn collect_songs_ci(expr: &crate::navidrome::search::Expr, navi: &NaviData) -> Vec<Song> {
    let mut matches = Vec::new();
    for album in &navi.album_list {
        for song in &get_album_songs(navi, album).await {
            if expr_matches_song_ci(expr, song) {
                matches.push(song.clone());
            }
        }
    }
    matches
}

async fn handle_search(args: FindArgs, kind: &str, navi: &NaviData) -> String {
    let expr = match parser::parse_filter(&args.filter) {
        Some(e) => e,
        None => return format!("ACK [2@0] {{{}}} could not parse filter\n", kind),
    };
    let mut matches = collect_songs_ci(&expr, navi).await;
    apply_sort(&mut matches, &args.sort);
    apply_window(&mut matches, args.window_start, args.window_end);
    let mut out = String::new();
    for song in &matches {
        out.push_str(&format_song(song));
    }
    out.push_str("OK\n");
    out
}
/* The virtual filesystem is /album/song. */
pub async fn database_handle(command: DatabaseStatus, client: &Client, navi: &NaviData) -> String {
    match command {
        DatabaseStatus::AlbumArt(id, ost) => {
            String::from_utf8_lossy(&art::return_album_art(&id, ost, client, &navi.config).await)
                .into_owned()
        }
        DatabaseStatus::Count(filter_str, group_type) => {
            let expr = match parser::parse_filter(&filter_str) {
                Some(e) => e,
                None => return "ACK [2@0] {count} could not parse filter\n".to_string(),
            };

            let mut total_songs = 0u32;
            let mut total_playtime = 0u32;
            let mut groups: HashMap<String, (u32, u32)> = HashMap::new();
            let has_group = !group_type.is_empty();

            for album in &navi.album_list {
                for song in &get_album_songs(&navi, album).await {
                    if expr.matches_song(song) {
                        total_songs += 1;
                        total_playtime += song.duration;
                        if has_group {
                            let entry = groups
                                .entry(song_group_value(song, &group_type))
                                .or_insert((0, 0));
                            entry.0 += 1;
                            entry.1 += song.duration;
                        }
                    }
                }
            }

            let mut out = format!("songs: {}\nplaytime: {}\n", total_songs, total_playtime);
            if has_group {
                let mut group_vec: Vec<_> = groups.into_iter().collect();
                group_vec.sort_by(|a, b| a.0.cmp(&b.0));
                for (key, (sc, pt)) in &group_vec {
                    out.push_str(&format!(
                        "songs: {}\nplaytime: {}\ngroup: {}\n",
                        sc, pt, key
                    ));
                }
            }
            out.push_str("OK\n");
            out
        }
        DatabaseStatus::Find(args) => handle_find(args, "find", &navi).await,
        DatabaseStatus::FindAdd(args) => handle_find(args, "findadd", &navi).await,

        DatabaseStatus::List(args) => {
            let expr = match args.filter.as_deref().filter(|filter| !filter.is_empty()) {
                Some(filter) => match parser::parse_filter(filter) {
                    Some(expr) => expr,
                    None => return "ACK [2@00] {list} could not parse filter\n".to_string(),
                },
                None => Expr::Empty,
            };
            let songs = collect_songs(&expr, &navi).await;
            let tag_label = capitalize_first(args.tag_type.as_deref().unwrap_or(""));

            if args.group_types.is_empty() {
                let mut values: Vec<String> = songs
                    .iter()
                    .map(|s| song_tag_value(s, args.tag_type.as_deref().unwrap_or("")))
                    .filter(|v| !v.is_empty())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                values.sort();

                let start = args.window_start.unwrap_or(0) as usize;
                let end = args
                    .window_end
                    .map(|value| value as usize)
                    .unwrap_or(values.len())
                    .min(values.len());
                if start < values.len() && end > start {
                    values = values[start..end].to_vec();
                } else {
                    values.clear();
                }

                let mut out = String::new();
                for v in &values {
                    out.push_str(&format!("{}: {}\n", tag_label, v));
                }
                out.push_str("OK\n");
                out
            } else {
                let mut out = String::new();
                for song in &songs {
                    let val = song_tag_value(song, args.tag_type.as_deref().unwrap_or(""));
                    if val.is_empty() {
                        continue;
                    }
                    for g in &args.group_types {
                        let gv = song_group_value(song, g);
                        if gv.is_empty() {
                            continue;
                        }
                        out.push_str(&format!("{}: {}\nGroup: {}\n", tag_label, val, gv));
                    }
                }
                out.push_str("OK\n");
                out
            }
        }
        // List every virtual file.
        DatabaseStatus::ListAll(_x) => {
            let mut new = String::new();
            for album in &navi.album_list {
                let songs = get_album_songs(&navi, album).await;
                for song in &songs {
                    new.push_str(&format!("file: {}/{}\n", album.name, song.path));
                }
            }
            new.push_str("OK\n");
            new
        }
        // List every file with metadata.
        DatabaseStatus::ListAllInfo(_x) => {
            let mut new = String::new();
            for album in &navi.album_list {
                let songs = get_album_songs(&navi, album).await;
                for song in &songs {
                    new.push_str(&format!("file: {}/{}\n", album.name, song.path));
                    new.push_str(&format!("duration: {}/{}\n", album.duration, song.path));
                    new.push_str(&format!("created: {}/{}\n", album.created, song.path));
                    new.push_str(&format!("artist: {}/{}\n", album.artist, song.path));
                    new.push_str(&format!(
                        "file: {}/{}\n",
                        album.year.unwrap_or(0),
                        song.path
                    ));
                }
            }
            new.push_str("Ok\n");
            new
        }
        #[allow(unused_variables)]
        DatabaseStatus::LsInfo(str) => {
            let parts: Vec<String> = str.split('/').map(String::from).collect();

            if let Some(name) = parts.first() {
                if !navi.album_list.iter().any(|a| a.name == *name) {
                    return "ACK [2@0] {lsinfo} No such album\n".to_string();
                }
            }
            if let Some(song_path) = parts.get(1) {
                if let Some(album) = navi.album_list.iter().find(|a| a.name == parts[0]) {
                    let songs = get_album_songs(&navi, album).await;
                    if let Some(song) = songs.iter().find(|s| s.path == *song_path) {
                        return format!("file: {}/{}\nOK\n", album.name, song.path);
                    }
                }
                return "ACK [2@0] {lsinfo} No such song\n".to_string();
            }

            if let Some(album) = navi.album_list.iter().find(|a| a.name == parts[0]) {
                let mut out = String::new();
                for song in get_album_songs(&navi, album).await {
                    out.push_str(&format!("file: {}/{}\n", album.name, song.path));
                }
                out.push_str("OK\n");
                return out;
            }

            format!("ACK [2@0] No such directory\n")
        }
        DatabaseStatus::ReadComment(path) => {
            let parts: Vec<&str> = path.split('/').collect();
            let Some(song_key) = parts.get(1) else {
                return "ACK [2@0] {readcomment} invalid path\n".to_string();
            };
            let Some(songs) = navi.songs_cache.get(*song_key) else {
                return "OK\n".to_string();
            };
            let Some(title) = parts.get(2) else {
                return "ACK [2@0] {readcomment} invalid path\n".to_string();
            };
            songs
                .iter()
                .find(|song| song.title == *title)
                .map(|song| format!("Comment: {}\nOK\n", song.comment))
                .unwrap_or_else(|| "OK\n".to_string())
        }
        DatabaseStatus::ReadPicture((path, offset)) => {
            let parts: Vec<&str> = path.split('/').collect();
            let Some(song_key) = parts.get(1) else {
                return "ACK [2@0] {readpicture} invalid path\n".to_string();
            };
            let Some(title) = parts.get(2) else {
                return "ACK [2@0] {readpicture} invalid path\n".to_string();
            };
            let Some(songs) = navi.songs_cache.get(*song_key) else {
                return "ACK [2@0] {readpicture} no such song\n".to_string();
            };
            let Some(song) = songs.iter().find(|song| song.title == *title) else {
                return "ACK [2@0] {readpicture} no such song\n".to_string();
            };
            String::from_utf8_lossy(
                &art::return_album_art(&song.id, offset, client, &navi.config).await,
            )
            .into_owned()
        }

        DatabaseStatus::Search(args) => handle_search(args, "search", &navi).await,

        #[allow(unused_variables)]
        DatabaseStatus::Searchaddpi(str, args) => {
            let _handle = handle_find(args, "findadd", &navi).await;
            _handle
        }
        DatabaseStatus::Update() => {
            // do nothing because it's cached on every boot
            let str = String::new();
            str
        }
        DatabaseStatus::SearchAdd(args) => handle_search(args, "searchadd", &navi).await,

        _ => format!("ACK-!"),
    }
}
pub struct ListArgs {
    pub tag_type: Option<String>,
    pub filter: Option<String>,
    pub group_types: Vec<String>,
    pub window: Option<(u32, u32)>,
    pub window_start: Option<u32>,
    pub window_end: Option<u32>,
}
