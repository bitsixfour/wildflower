use clap::Parser;
use reqwest::Client;
use event_listener::{Event, Listener};

use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

mod navi;
mod song;
use crate::navi::{NaviData, SubsonicResponse};



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
    duration: f32,
}
/* Trait for actual Mpd and
 * the Navidrome api */
pub trait AlbumData {
    fn get_id(var: &str) -> String;
    fn get_dur(var: &str) -> String;
}





#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting ze mpd server....");

    let client: Client = reqwest::Client::new();

    let res: SubsonicResponse = navi::navi_obj(&client).await?;
    let navi: NaviData = NaviData::new(res).await;

    
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_client(socket));
    }
    Ok(())
}
async fn handle_client(socket: tokio::net::TcpStream) {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();


    println!("dbg see if this works");
    writer.write_all(b"OK MPD 67.67.67\n").await.unwrap();
    
    while let Some(line) = lines.next_line().await.unwrap() {
        match line.trim() {
            "play" => {
                // start playback somehow
            }
            "pause" => {
                // pause playback somehow
            }
            "currentsong" => {
                mpd::getSong();
            }
            _ => {
                writer.write_all(b"OK\n").await.unwrap();
            }
        }
    }
}
