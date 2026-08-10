use std::collections::HashSet;
use std::time::Duration;

use crate::navidrome::navi::NaviData;
use crate::play::playback::{AudioState, SharedState};
use crate::play::tracklist::Song;

#[allow(dead_code)]
pub enum QueueHandle {
    ClearError,
    CurrentSong,
    Idle(Vec<String>),
    Status,
    Stats,
    Consume(bool),
    Random(bool),
    Repeat(bool),
    SetVol(u32),
    GetVol,
}

pub async fn queue_handle(hdle: QueueHandle, state: &SharedState, navi: &NaviData) -> String {
    match hdle {
        QueueHandle::ClearError => "OK\n".to_string(),
        QueueHandle::CurrentSong => current_song(state, navi).await,
        QueueHandle::Idle(_) => "OK\n".to_string(),
        QueueHandle::Status => status(state).await,
        QueueHandle::Stats => stats(navi),
        QueueHandle::Consume(b) => set_field(state, |s| s.consume = b).await,
        QueueHandle::Random(b) => set_field(state, |s| s.random = b).await,
        QueueHandle::Repeat(b) => set_field(state, |s| s.repeat = b).await,
        QueueHandle::SetVol(v) => {
            let clamped = v.min(100) as i32;
            set_field(state, move |s| s.volume = clamped).await
        }
        QueueHandle::GetVol => {
            let st = state.read().await;
            format!("volume: {}\nOK\n", st.volume)
        }
    }
}

async fn set_field<F>(state: &SharedState, f: F) -> String
where
    F: FnOnce(&mut crate::play::playback::PlayerState),
{
    let mut st = state.write().await;
    f(&mut st);
    "OK\n".to_string()
}

async fn status(state: &SharedState) -> String {
    let st = state.read().await;
    let mut out = String::new();
    out.push_str(&format!("volume: {}\n", st.volume));
    out.push_str(&format!("repeat: {}\n", st.repeat as i32));
    out.push_str(&format!("random: {}\n", st.random as i32));
    out.push_str(&format!("single: {}\n", st.single as i32));
    out.push_str(&format!("consume: {}\n", st.consume as i32));
    out.push_str(&format!("playlist: {}\n", st.playlist_version));
    out.push_str(&format!("playlistlength: {}\n", st.playlist_length));
    out.push_str(&format!(
        "state: {}\n",
        match st.state {
            AudioState::Play => "play",
            AudioState::Stop => "stop",
            AudioState::Pause => "pause",
        }
    ));
    if let Some(pos) = st.song_pos {
        out.push_str(&format!("song: {}\n", pos));
    }
    if let Some(id) = &st.song_id {
        out.push_str(&format!("songid: {}\n", id));
    }
    out.push_str(&format!("elapsed: {:.3}\n", st.elapsed.as_secs_f64()));
    if st.duration > Duration::from_secs(0) {
        out.push_str(&format!("duration: {:.3}\n", st.duration.as_secs_f64()));
    }
    out.push_str("OK\n");
    out
}

async fn current_song(state: &SharedState, navi: &NaviData) -> String {
    let st = state.read().await;
    let mut out = String::new();
    let Some(song_id) = &st.song_id else {
        out.push_str("OK\n");
        return out;
    };
    match find_song_by_id(navi, song_id) {
        Some(song) => push_song(&mut out, &song),
        None => {
            out.push_str(&format!("file: {}\n", song_id));
            out.push_str(&format!("Id: {}\n", song_id));
        }
    }
    out.push_str("OK\n");
    out
}

fn push_song(out: &mut String, song: &Song) {
    out.push_str(&format!("file: {}\n", song.path));
    out.push_str(&format!("Title: {}\n", song.title));
    out.push_str(&format!("Artist: {}\n", song.artist));
    out.push_str(&format!("Album: {}\n", song.album));
    out.push_str(&format!("Time: {}\n", song.duration));
    if !song.id.is_empty() {
        out.push_str(&format!("Id: {}\n", song.id));
    }
}
// TODO: make this into a trait because another struct uses it
fn find_song_by_id(navi: &NaviData, song_id: &str) -> Option<Song> {
    for songs in navi.albums_cache.values() {
        for song in songs {
            if song.id == song_id {
                return Some(song.clone());
            }
        }
    }
    None
}

fn stats(navi: &NaviData) -> String {
    let mut artists: HashSet<String> = HashSet::new();
    for album in &navi.album_list {
        if !album.artist.is_empty() {
            artists.insert(album.artist.to_lowercase());
        }
    }
    let mut songs: u32 = 0;
    let mut playtime: u64 = 0;
    for songs_in_album in navi.albums_cache.values() {
        songs += songs_in_album.len() as u32;
        for song in songs_in_album {
            playtime += song.duration as u64;
        }
    }
    let uptime = "NaN";
    format!(
        "artists: {}\nalbums: {}\nsongs: {}\nuptime: {}\n\
         db_playtime: {}\ndb_update: 0\nplaytime: 0\nOK\n",
        artists.len(),
        navi.album_list.len(),
        songs,
        uptime,
        playtime,
    )
}
