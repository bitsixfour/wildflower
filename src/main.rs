mod navidrome;
mod config;
mod play;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use reqwest::Client;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::navidrome::database::{self, DatabaseStatus, ListArgs, FindArgs};
use crate::navidrome::navi::NaviData;
use crate::play::playback::{find_song_by_uri, find_songs_by_uri, CurrentSong, PlaybackStatus, PlayerState, AudioState, SharedState};
use crate::play::queue::{QueueHandle, queue_handle};

const PORT: u32 = 6600;


pub trait BytesAlbum {
    
    async fn return_album_art(req: &str, 
        return_offset: i64) -> Vec<u8>;  

}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?; // 6600 is where MPD lives
    println!("We are ze running at port {PORT}");
    let heckin_reqwest: Client = reqwest::Client::new();

    let navi: NaviData = match NaviData::load_or_fetch(&heckin_reqwest, Path::new("wildflower-cache.json")).await {
        Ok(n) => {
            let album_count = n.album_list.len();
            let song_count: usize = n.albums_cache.values().map(|v| v.len()).sum();
            n
        }
        Err(e) => {
            eprintln!("navidrome + cache both unavailable: {e:#}. starting with empty library.");
            NaviData::init_empty()
        }
    };
 

    let shared_state: SharedState = Arc::new(tokio::sync::RwLock::new(PlayerState {
        volume: 100,
        state: AudioState::Stop,
        song_pos: None,
        song_id: None,
        elapsed: Duration::from_secs(0),
        duration: Duration::from_secs(0),
        playlist_length: 0,
        playlist_version: 0,
        repeat: false,
        random: false,
        single: false,
        consume: false,
    }));

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<PlaybackStatus>(100);

    let engine_state = Arc::clone(&shared_state);
    let heckin_reqwes = heckin_reqwest.clone();
    tokio::spawn(async move {
        let mut engine = CurrentSong::new(&heckin_reqwest).await;
        while let Some(cmd) = cmd_rx.recv().await {
            engine.handle(cmd, &heckin_reqwest).await;
            let mut st = engine_state.write().await;
            let pos: i32 = engine.queue.cursor;
            st.state = AudioState::Play; 
            st.song_pos = pos.try_into().ok();
            st.playlist_length = engine.queue.items.len();
        }
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let client_tx = cmd_tx.clone();
        let client_state = Arc::clone(&shared_state);
        let navi = navi.clone();
        let reqwest = heckin_reqwes.clone();
        tokio::spawn(async move {
            init_client(socket, client_tx, client_state, navi, reqwest).await;
        });
    }
}

async fn init_client(socket: TcpStream, cmd_tx: tokio::sync::mpsc::Sender<PlaybackStatus>, state: SharedState, music_data: NaviData, client: Client) {
    let (reader, mut writer) = tokio::io::split(socket);
    let mut reader = BufReader::new(reader);
    let _ = writer.write_all(b"OK MPD 0.25.0\n").await;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim_end();
                if trimmed.is_empty() { continue; }
                let response = handle_case(trimmed, &cmd_tx, &state, &client, &music_data).await;
                let _ = writer.write_all(response.as_bytes()).await;
                if trimmed == "close" {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}



async fn handle_case(input: &str, cmd_tx: &tokio::sync::mpsc::Sender<PlaybackStatus>, state: &SharedState, client: &Client, navi: &NaviData) -> String {
    let trimmed = input.trim();
    let mut parts: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();

    for c in trimmed.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    let cmd = parts.get(0).map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "play" => {
            let _ = cmd_tx.send(PlaybackStatus::Play).await;
            "OK\n".to_string()
        }
        "pause" => {
            let arg: i32 = parts.get(1).unwrap().parse().unwrap();
            match arg {
                0..1 =>  {
                    let _ = cmd_tx.send(PlaybackStatus::Pause(arg)).await;
                    "Ok\n".to_string()
                }
                _ => {
                    "ACK\n".to_string()
                }
            }
        }
        "playid" => {
            let arg = parts.get(1).unwrap().parse::<usize>().unwrap() as usize;
            let _ = cmd_tx.send(PlaybackStatus::PlayPos(arg)).await;
            "OK\n".to_string()
        }
        "next" => {
            let _ = cmd_tx.send(PlaybackStatus::Next()).await;
            "OK\n".to_string()
        }
        "previous" => {
            let _ = cmd_tx.send(PlaybackStatus::Previous).await;
            "OK\n".to_string()
        }
        "seek" => {
            let songpos: u64 = parts.get(1).unwrap().parse::<u64>().unwrap();
            let time: String = parts.get(2).unwrap().parse::<String>().unwrap();
            let turp = (songpos, time);
            let _ = cmd_tx.send(PlaybackStatus::SeekId(turp)).await;
            "Ok\n".to_string()
        }
        "seekid" => {
            let songpos: u64 = parts.get(1).unwrap().parse::<u64>().unwrap();
            let time: String = parts.get(2).unwrap().parse::<String>().unwrap();
            let turp = (songpos, time);
            let _ = cmd_tx.send(PlaybackStatus::SeekId(turp)).await;
            "Ok\n".to_string()
        }
        "seekcur" => {
            let dur = parts.get(1).unwrap().parse::<u64>().unwrap();
            let _ = cmd_tx.send(PlaybackStatus::SeekCur(dur)).await;
            "Ok\n".to_string()
        }
        "stop" => {
            let _ = cmd_tx.send(PlaybackStatus::Stop).await;
            "Ok\n".to_string()
        }
        "add" => {
            let uri = parts.get(1).cloned().unwrap_or_default();
            let songs = find_songs_by_uri(navi, &uri);
            if songs.is_empty() {
                return "ACK [2@0] {add} no such file or directory\n".to_string();
            }
            for song in songs {
                let _ = cmd_tx.send(PlaybackStatus::Add(song)).await;
            }
            "OK\n".to_string()
        }
        "addid" => {
            let uri = parts.get(1).cloned().unwrap_or_default();
            let pos = parts.get(2).and_then(|s| s.parse::<usize>().ok());
            let Some(song) = find_song_by_uri(navi, &uri) else {
                return "ACK [2@0] {addid} no such song\n".to_string();
            };
            let id = song.id.clone();
            let _ = cmd_tx.send(PlaybackStatus::AddId(song, pos)).await;
            format!("Id: {}\nOK\n", id)
        }
        "status" => queue_handle(QueueHandle::Status, state, navi).await,
        "currentsong" => queue_handle(QueueHandle::CurrentSong, state, navi).await,
        "playlistinfo" => "OK\n".to_string(),
        "list" => {
            if parts.len() < 2 {
                return "ACK [2@0] {list} missing tag type\n".to_string();
            }

            let tag_type = parts[1].clone();
            let mut i = 2;
            let mut filter_parts: Vec<String> = Vec::new();
            let mut group_types: Vec<String> = Vec::new();
            let mut window: Option<(u32, u32)> = None;

            while i < parts.len() {
                match parts[i].as_str() {
                    "group" => {
                        i += 1;
                        if i < parts.len() {
                            group_types.push(parts[i].clone());
                        }
                    }
                    "window" => {
                        i += 1;
                        if i < parts.len() {
                            if let Some((s, e)) = parts[i].split_once(':') {
                                let start = s.parse().unwrap_or(0);
                                let end = e.parse().unwrap_or(u32::MAX);
                                window = Some((start, end));
                            }
                        }
                    }
                    _ => {
                        filter_parts.push(parts[i].clone());
                    }
                }
                i += 1;
            }

            let filter = if filter_parts.is_empty() {
                None
            } else {
                Some(filter_parts.join(" "))
            };

            let list_args = ListArgs {
                tag_type: Some(tag_type),
                filter,
                group_types,
                window,
                window_start: window.map(|(s, _)| s),
                window_end: window.map(|(_, e)| e),
            };

            database::database_handle(
                DatabaseStatus::List(list_args),
                client,
                &navi,
            )
            .await
        }
        // music database
        "albumart" => {
            let input: String = parts.get(1).unwrap_or(&String::new()).clone();
            let offset: i64 = parts.get(2)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            let args: DatabaseStatus = DatabaseStatus::AlbumArt(input, offset);
            let res: String = database::database_handle(args, client, navi).await;
            res
        }
        "count" => {
            if parts.len() < 2 {
                return "ACK [2@0] {count} missing filter\n".to_string();
            }
            let filter = parts[1].clone();
            let mut group = String::new();
            let mut i = 2;
            while i < parts.len() {
                if parts[i] == "group" && i + 1 < parts.len() {
                    group = parts[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            database::database_handle(DatabaseStatus::Count(filter, group), client, navi).await
        }
        "find" => {
            if parts.len() < 2 {
                return "ACK [2@0] {find} missing filter\n".to_string();
            }
            let fltr = parts[1].clone();
            let mut sort = None;
            let mut window_start = None;
            let mut window_end = None;
            let mut i = 2;
            while i < parts.len() {
                match parts[i].as_str() {
                    "sort" if i + 1 < parts.len() => {
                        sort = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "window" if i + 1 < parts.len() => {
                        if let Some((s, e)) = parts[i + 1].split_once(':') {
                            window_start = s.parse().ok();
                            window_end = e.parse().ok();
                        }
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            database::database_handle(
                DatabaseStatus::Find(FindArgs {
                    filter: fltr, sort, window_start, window_end, position: None,
                }),
                client, navi,
            ).await
        }
        "findadd" => {
            if parts.len() < 2 {
                return "ACK [2@0] {findadd} missing filter\n".to_string();
            }
            let fltr = parts[1].clone();
            let mut sort = None;
            let mut window_start = None;
            let mut window_end = None;
            let mut i = 2;
            while i < parts.len() {
                match parts[i].as_str() {
                    "sort" if i + 1 < parts.len() => {
                        sort = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "window" if i + 1 < parts.len() => {
                        if let Some((s, e)) = parts[i + 1].split_once(':') {
                            window_start = s.parse().ok();
                            window_end = e.parse().ok();
                        }
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            let _ = database::database_handle(
                DatabaseStatus::FindAdd(FindArgs {
                    filter: fltr, sort, window_start, window_end, position: None,
                }),
                client, navi,
            ).await;
            // TODO: ADD HELPER FUNCTION TO ADD SONG (when I actually add "add")
            "".to_string()
        }
        "listall" => {
            let path = parts.get(1).cloned().unwrap_or_default();
            database::database_handle(DatabaseStatus::ListAll(Box::new(path)), client, navi).await
        }
        "listallinfo" => {
            let path = parts.get(1).cloned().unwrap_or_default();
            database::database_handle(DatabaseStatus::ListAllInfo(Box::new(path)), client, navi).await
        }
        "lsinfo" => {
            let path = parts.get(1).cloned().unwrap_or_default();
            database::database_handle(DatabaseStatus::LsInfo(path), client, navi).await
        }
        "readcomment" => {
            let path = parts.get(1).cloned().unwrap_or_default();
            database::database_handle(DatabaseStatus::ReadComment(path), client, navi).await
        }
        "readpicture" => {
            let path = parts.get(1).cloned().unwrap_or_default();
            let offset: i64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            database::database_handle(DatabaseStatus::ReadPicture((path, offset)), client, navi).await
        }
        "search" => {
            if parts.len() < 2 {
                return "ACK [2@0] {search} missing filter\n".to_string();
            }
            let fltr = parts[1].clone();
            let mut sort = None;
            let mut window_start = None;
            let mut window_end = None;
            let mut i = 2;
            while i < parts.len() {
                match parts[i].as_str() {
                    "sort" if i + 1 < parts.len() => {
                        sort = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "window" if i + 1 < parts.len() => {
                        if let Some((s, e)) = parts[i + 1].split_once(':') {
                            window_start = s.parse().ok();
                            window_end = e.parse().ok();
                        }
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            database::database_handle(
                DatabaseStatus::Search(FindArgs {
                    filter: fltr, sort, window_start, window_end, position: None,
                }),
                client, navi,
            ).await
        }
        "searchadd" => {
            if parts.len() < 2 {
                return "ACK [2@0] {searchadd} missing filter\n".to_string();
            }
            let fltr = parts[1].clone();
            let mut sort = None;
            let mut window_start = None;
            let mut window_end = None;
            let mut i = 2;
            while i < parts.len() {
                match parts[i].as_str() {
                    "sort" if i + 1 < parts.len() => {
                        sort = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "window" if i + 1 < parts.len() => {
                        if let Some((s, e)) = parts[i + 1].split_once(':') {
                            window_start = s.parse().ok();
                            window_end = e.parse().ok();
                        }
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            let _ = database::database_handle(
                DatabaseStatus::SearchAdd(FindArgs {
                    filter: fltr, sort, window_start, window_end, position: None,
                }),
                client, navi,
            ).await;
            "".to_string()
        }
        "searchaddpl" => {
            if parts.len() < 3 {
                return "ACK [2@0] {searchaddpl} missing arguments\n".to_string();
            }
            let name = parts[1].clone();
            let fltr = parts[2].clone();
            let mut sort = None;
            let mut window_start = None;
            let mut window_end = None;
            let mut i = 3;
            while i < parts.len() {
                match parts[i].as_str() {
                    "sort" if i + 1 < parts.len() => {
                        sort = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "window" if i + 1 < parts.len() => {
                        if let Some((s, e)) = parts[i + 1].split_once(':') {
                            window_start = s.parse().ok();
                            window_end = e.parse().ok();
                        }
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            database::database_handle(
                DatabaseStatus::Searchaddpi(name, FindArgs {
                    filter: fltr, sort, window_start, window_end, position: None,
                }),
                client, navi,
            ).await
        }
        "update" => {
            let _ = parts.get(1);
            database::database_handle(DatabaseStatus::Update(), client, navi).await
        }
        // most clients don't use this at all and it isn't even enabled by default
        "getfingerprint" => {
            "".to_string()
        }
        "ping" => "OK\n".to_string(),
        "close" => "OK\n".to_string(),
        _ => format!("ACK [2@0] {{unknown command}} {}\n", cmd),
    }
}
