use clap::Parser;
use reqwest::Client;
use event_listener::{Event, Listener};
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

mod navi;
mod tracklist;
mod playback;
mod search;
mod parser;
use crate::navi::{NaviData, SubsonicResponse};
use crate::playback::{CurrentSong, PlaybackStatus};




const PORT: i32 = 6600;



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'l', long)]
    album: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

pub struct MpdSong {
    id: String,
    title: String,
    artist: String,
    album: String,
    // length: i32,
}

/* Trait for actual Mpd and
 * the Navidrome api */
pub trait SubsonicParse {
    pub fn get_length() -> String;
    pub fn get_url() -> String;
    pub fn navi_to_song(var: &Song) -> MpdSong;


}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting ze mpd server....");
    let test_id: &str = "23M5Qz4SmDa79E5MR0woPr";
    let heckin_reqwest: Client = reqwest::Client::new();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?; // 6600 is where MPD lives
    println!("We are ze running at port {PORT}");

    let navi: NaviData = NaviData::init_empty();
    




    let (tx, mut rx) = tokio::sync::mpsc::channel::<PlaybackStatus>(100);
    tokio::spawn(async move {
        let mut engine = CurrentSong::new(&test_id, &heckin_reqwest).await.unwrap();
        while let Some(cmd) = rx.recv().await {
            engine.handle(cmd);
        }
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let tx = tx.clone();
        tokio::spawn(async move {
            init_client(socket, tx).await;
        });
    }
    Ok(())
}
async fn init_client(mut socket: TcpStream, tx: tokio::sync::mpsc::Sender<PlaybackStatus>) {
    let reader_socket = socket.try_clone().unwrap();
    let reader = BufReader::new(reader_socket);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = handle_case(&line);

        if socket.write_all(response.as_bytes()).await.is_err() {
            break;
        }

        if line.trim() == "close" {
            break;
        }
    }
}
/* THE actual handling of mpd jorunal reqwests or whatever */
async fn handle_case(
