use std::time::Duration;
use std::sync::Arc;
use bytes::Bytes;
use reqwest::Client;
use std::io::Cursor;
use rodio::{Decoder, Player, MixerDeviceSink};
use crate::play::tracklist::Song;

const URL: &str = "192.168.1.20:8097";

#[allow(dead_code)]
pub struct CurrentSong {
    pub song_id: String,
    pub stream: Bytes,
    var: MixerDeviceSink,
    pub queue: PlaybackQueue,
}
pub struct PlaybackQueue {
    pub items: Vec<Song>,
    pub cursor: i32,
    player: Player,
}


#[allow(dead_code)]
pub enum PlaybackStatus {
    Seek((u64, String)),
    Next(),
    Pause(i32),
    Play,           
    PlayPos(usize),    
    PlayId(String),
    Previous,
    SeekId((u64, String)),
    SeekCur(u64),
    Stop,
    Add(Song),
    AddId(Song, Option<usize>),
}

#[allow(dead_code)]
pub enum QueueStatus {
    Add(String, i32),
    AddId(String, i32),
    Clear(),
    Delete(String),
    DeleteId(String),
    // parse regex for this
    Move(String),
    MoveId(String, String),
    // Playlist(),
    Playlistfind(String, String),
    PlaylistId(String),
    PlaylistInfo(String),
    PlaylistSearch(String),
    PiChanges(String, (i32, i32)),
    PiChangesPos(String, (i32, i32)),
    Prio(i32, (i32, i32)),
    PrioId(i32, (i32, i32))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AudioState {
    Play,
    Stop,
    Pause,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: i32,
    pub state: AudioState,
    pub song_pos: Option<usize>,
    pub song_id: Option<String>,
    pub elapsed: Duration,
    pub duration: Duration,
    pub playlist_length: usize,
    pub playlist_version: u32,
    pub repeat: bool,
    pub random: bool,
    pub single: bool,
    pub consume: bool,
}
pub type SharedState = Arc<tokio::sync::RwLock<PlayerState>>;

impl CurrentSong {
     pub async fn new(_client: &Client) -> Self {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let mixer = handle.mixer().clone();
        Self {
            song_id: String::new(),
            var: handle,
            stream: Bytes::new(),
            queue: PlaybackQueue {
                items: Vec::new(),
                cursor: 0,
                player: Player::connect_new(&mixer),
            }
        }
    }
    pub async fn handle(&mut self, command: PlaybackStatus, client: &Client) {
        match command {
            PlaybackStatus::Next() => {
                println!("dbg... we're going to the next song...");
                self.queue.next(client).await;
            }
            PlaybackStatus::Pause(io) => {
                println!("pause");
                match io {
                    0 => {
                        self.queue.player.pause();
                    }
                    1 => {
                        self.queue.player.play();
                    }
                    _ => {
                        println!("unexpected args..");
                    }
                }
            }
            PlaybackStatus::Play => {
                println!("play (resume)");
                self.queue.rebuild_buffer(client).await;
                self.queue.player.play();
            }
            PlaybackStatus::PlayPos(pos) => {
                println!("play pos {}", pos);
                let dur = Duration::from_secs(pos as u64);
                self.queue.player.try_seek(dur).unwrap();
            }
            #[allow(unused_variables)]
            PlaybackStatus::PlayId(_io) => {
                println!("play by id");
                self.queue.player.stop();
                for idx in 0..self.queue.items.len() {
                    if let Some(_item) = self.queue.items.get(idx) {
                        println!("match id");
                        for itr in 0..idx {
                            println!("{itr}");
                            self.queue.player.skip_one();
                        }
                    }
                }
                self.queue.player.play();
            }
            PlaybackStatus::Previous => {
                println!("previous");
                self.queue.previous(client).await;
            }
            PlaybackStatus::Seek(io) => {
                let pos_seek = io.0.clone();
                self.queue.jump_to(client, pos_seek as i32).await;
            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekId(id) => {
                let sec_seek = Duration::from_secs(id.0);
                for idx in 0..self.queue.items.len() {
                    if let Some(item_id) = self.queue.items.get(idx).map(|s| s.id.clone()) {
                        let _ = item_id;
                        self.queue.jump_to(client, idx as i32).await;
                        self.queue.player.try_seek(sec_seek).unwrap();
                    }
                }
            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekCur(io) => {
                let var = self.queue.player.get_pos();
                let delta = Duration::from_secs(io);
                for idx in 0..self.queue.items.len() {
                    if let Some(_) = self.queue.items.get(idx) {
                        self.queue.jump_to(client, idx as i32).await;
                        let _ = self.queue.player.try_seek(var + delta);
                    }
                }
            }
            PlaybackStatus::Stop => {
                println!("stop!");
                self.queue.player.stop();
            }
            PlaybackStatus::Add(song) => {
                self.queue.items.push(song);
            }
            PlaybackStatus::AddId(song, pos) => {
                let p = pos.unwrap_or(self.queue.items.len()).min(self.queue.items.len());
                self.queue.items.insert(p, song);
            }
        }
    }




    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }

}

#[allow(unused_variables, dead_code)]
impl PlaybackQueue {

    pub async fn next(&mut self, client: &Client) {
        if self.items.is_empty() { return; }
        if (self.cursor as usize) + 1 >= self.items.len() { return; }
        self.player.skip_one();
        self.cursor += 1;
        if let Some(s) = self.fetch_and_decode(self.cursor as usize + 1, client).await {
            self.player.append(s);
        }
    }

    pub async fn previous(&mut self, client: &Client) {
        if self.items.is_empty() || self.cursor == 0 { return; }
        self.cursor -= 1;
        self.player.stop();
        self.player.clear();
        self.rebuild_buffer(client).await;
        self.player.play();
    }

    pub async fn remove(&mut self, client: &Client, idx: i32) {
        let rm = idx as usize;
        self.items.remove(rm);
    }

    pub async fn jump_to(&mut self, client: &Client, idx: i32) {
        self.cursor = idx;
        self.rebuild_buffer(client).await;
    }


    async fn sink_init(&mut self, stream: Vec<u8>, client: &Client) {
        let source = Decoder::new(Cursor::new(stream)).unwrap();
        self.player.append(source);
        self.rebuild_buffer(client).await;
        self.player.play();
    }

    async fn destroy_buffer(&mut self) {
        self.player.clear();
    }
    async fn rebuild_buffer(&mut self, client: &Client) {
        self.player.clear();
        if let Some(s) = self.fetch_and_decode(self.cursor as usize, client).await {
            self.player.append(s);
        }
        if let Some(s) = self.fetch_and_decode(self.cursor as usize + 1, client).await {
            self.player.append(s);
        }
    }
    async fn fetch_and_decode(&self, idx: usize, client: &Client) -> Option<Decoder<Cursor<Vec<u8>>>> {
        let song = self.items.get(idx)?;
        let bytes = self.get_audio_stream(&song.id, client).await;
        Decoder::new(Cursor::new(bytes)).ok()
    }
    async fn get_audio_stream(&self, search_id: &str, client: &Client) -> Vec<u8> {
        let req = format!("http://nix:2008@192.168.1.20:8097/rest/stream.view?u=nix&p=2008&v=1.16.1& c=app&id={}", search_id);
        let mut vec: Vec<u8> = Vec::new();
        let mut bytes: Vec<u8> = reqwest::Client::new()
            .get(req)
            .send().await.unwrap()
            .error_for_status().unwrap()
            .bytes().await.unwrap()
            .to_vec();
        vec.append(&mut bytes);
        vec
        }
}

use crate::navidrome::navi::NaviData;

fn mpd_path(album_name: &str, song_path: &str) -> String {
    format!("{}/{}", album_name, song_path)
}

pub fn find_song_by_uri(navi: &NaviData, uri: &str) -> Option<Song> {
    for album in &navi.album_list {
        for song in navi.albums_cache.get(&album.id).map(|v| v.as_slice()).unwrap_or(&[]) {
            if mpd_path(&album.name, &song.path) == uri || song.path == uri {
                return Some(song.clone());
            }
        }
    }
    None
}

pub fn find_songs_by_uri(navi: &NaviData, uri: &str) -> Vec<Song> {
    let mut out = Vec::new();
    for album in &navi.album_list {
        if album.name == uri {
            if let Some(songs) = navi.albums_cache.get(&album.id) {
                return songs.clone();
            }
        }
    }
    if let Some(song) = find_song_by_uri(navi, uri) {
        out.push(song);
        return out;
    }
    for album in &navi.album_list {
        for song in navi.albums_cache.get(&album.id).map(|v| v.as_slice()).unwrap_or(&[]) {
            if song.path.starts_with(uri) {
                out.push(song.clone());
            }
        }
    }
    out
}
