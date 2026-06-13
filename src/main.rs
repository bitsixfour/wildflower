use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use reqwest::Client;
use event_listener::{Event, Listener};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

mod navi;
mod tracklist;
mod playback;
mod search;
mod parser;
mod audio;
// mod rodio_stub;
use crate::navi::{NaviData, SubsonicResponse};
use crate::tracklist::Song;
use crate::playback::{CurrentSong, PlaybackStatus, PlayerState, AudioState, SharedState};




const PORT: i32 = 6600;



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'l', long)]
    album: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}
#[allow(dead_code)]
pub struct MpdSong {
    id: String,
    title: String,
    artist: String,
    album: String,
    // length: i32,
}

/* Trait for actual Mpd and
pub trait SubsonicParse {
    fn get_length() -> String;
    fn get_url() -> String;
    fn navi_to_song(var: &Song) -> MpdSong;


}
*/



#[tokio::main]
#[allow(unused_variables)]
async fn main() -> anyhow::Result<()> {
    println!("starting ze mpd server....");
    let test_id: &str = "23M5Qz4SmDa79E5MR0woPr";
    let heckin_reqwest: Client = reqwest::Client::new();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?; // 6600 is where MPD lives
    println!("We are ze running at port {PORT}");

    let navi: NaviData = NaviData::init_empty();
    

    let shared_state: SharedState = Arc::new(tokio::sync::RwLock::new(PlayerState {
        volume: 100,
        state: AudioState::Stop,
        song_pos: None,
        song_id: None,
        elapsed: Duration::from_secs(0),
        duration: Duration::from_secs(0),
        playlist_length: 0,
        playlist_version: 0,
    }));

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<PlaybackStatus>(100);






    let engine_state = Arc::clone(&shared_state);
    tokio::spawn(async move {
        let mut engine = CurrentSong::new(&test_id, heckin_reqwest).await;
        while let Some(cmd) = cmd_rx.recv().await {
            engine.handle(cmd);
            let mut st = engine_state.write().await;
            st.state = AudioState::Play; 
            st.song_pos = Some(engine.get_queue_cursor() as usize);
            st.playlist_length = engine.get_queue_len();
        }
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let client_tx = cmd_tx.clone();
        let client_state = Arc::clone(&shared_state);
        tokio::spawn(async move {
            init_client(socket, client_tx, client_state, navi).await;
        });
    }
}

async fn init_client(socket: TcpStream, cmd_tx: tokio::sync::mpsc::Sender<PlaybackStatus>, state: SharedState, music_data: NaviData) {
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
                let response = handle_case(trimmed, &cmd_tx, &state).await;
                if writer.write_all(response.as_bytes()).await.is_err() {
                    break;
                }
                if trimmed == "close" {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}



/* Giant monolithic handling of all 50+ MPD functions. */
async fn handle_case(input: &str, cmd_tx: &tokio::sync::mpsc::Sender<PlaybackStatus>, state: &SharedState) -> String {
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



        /* Controlling playback segment on mpd.readthedocs.io */
        "play" => {
            // let arg = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
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
        "stop" => {
            let _ = cmd_tx.send(PlaybackStatus::Stop).await;
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
        /* The Queue (Section on MPD API SPEC 
         *
         */




        "status" => {
            let st = state.read().await;
            let mut out = String::new();
            out.push_str(&format!("volume: {}\n", st.volume));
            out.push_str("repeat: 0\n");
            out.push_str("random: 0\n");
            out.push_str("single: 0\n");
            out.push_str("consume: 0\n");
            out.push_str(&format!("playlist: {}\n", st.playlist_version));
            out.push_str(&format!("playlistlength: {}\n", st.playlist_length));
            out.push_str(&format!("state: {}\n", match st.state {
                AudioState::Play => "play",
                AudioState::Stop => "stop",
                AudioState::Pause => "pause",
            }));
            if let Some(pos) = st.song_pos {
                out.push_str(&format!("song: {}\n", pos));
            }
            if let Some(ref id) = st.song_id {
                out.push_str(&format!("songid: {}\n", id));
            }
            out.push_str(&format!("elapsed: {:.3}\n", st.elapsed.as_secs_f64()));
            if st.duration > Duration::from_secs(0) {
                out.push_str(&format!("duration: {:.3}\n", st.duration.as_secs_f64()));
            }
            out.push_str("OK\n");
            out
        }
        "currentsong" => {
            let st = state.read().await;
            let mut out = String::new();
            if let Some(ref id) = st.song_id {
                out.push_str(&format!("file: {}\n", id));
                out.push_str(&format!("Id: {}\n", id));
            }
            out.push_str("OK\n");
            out
        }
        "playlistinfo" => {
            let st = state.read().await;
            let mut out = String::new();
            out.push_str("OK\n");
            out
        }
        "ping" => "OK\n".to_string(),
        "close" => "OK\n".to_string(),

        
        // error...
        _ => format!("ACK [5@0] {{{cmd}}} unknown command\n"),
    }
}
