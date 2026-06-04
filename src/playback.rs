use std::time::Duration;


use reqwest::header::ALLOW;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, source::Source};
use rodio::Player;
use std::io::Cursor;
use crate::MpdSong;

const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    song_id: Arc<Mutex<String>>,
    var: MixerDeviceSink, // Player depends on Mixer (it says on rust document dont forget this)
    player: Player,
    queue: PlaybackQueue,
}
pub struct PlaybackQueue {
    items: Vec<Song>,
    cursor: i32,
}



pub enum PlaybackStatus {
    Seek(f32),
    Next(),
    Pause(i32),
    Play(Duration),
    PlayId(String),
    Previous,
    SeekId(&str),
    SeekCur(f32),
    Stop,
}


impl CurrentSong {
    pub async fn new(song_id: &str, client: &Client) -> Self {
        let le_url: String = CurrentSong::fmt_url(song_id);
        let bytes = client
            .get(&le_url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await?;
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let plr = rodio::Player::connect_new(&handle.mixer());
        Self {
            song_id: Arc::new(Mutex::new(format!(song_id))),
            var: handle,
            player: plr,
            queue: PlaybackQueue {
                items: Vec::new(),
                cursor: 0,
            }
        }

    }
    pub fn handle(&mut self,command: PlaybackStatus)  {
        match command {
            PlaybackStatus::Next() => {
                println!("dbg... we're going to the next song...");
                self.player.skip_one();
            }
            PlaybackStatus::Pause(io) => {
                println!("pause");
                match io {
                    0 => {
                        &self.player.pause();
                    }
                    1 => {
                        &self.player.play();
                    }
                    _ => {
                        println!("unexpected args..");
                    }
                }
            }
            PlaybackStatus::Play(io) => {
                println!("play");
                let delta = (io - &self.player.len());
                for _ in 0..delta { 
                    &self.player.skip_one();
                }

            }
            #[allow(unused_variables)]
            PlaybackStatus::PlayId(io) => {
                println!("play by id");
                let mut cnt;
                for idx in 0..self.queue.items.len() {
                    match &self.queue.items.get(idx) {
                        io => {
                            println!("match id");
                            for itr in 0..idx {
                                println!("{itr}");
                                &self.player.skip_one();
                            }
                        }
                    }
                }



            }
            PlaybackStatus::Previous => {
                println!("previous");
                &self.queue.cursor.saturating_sub(1);
                // finish when queue is more polished

            }
            PlaybackStatus::Seek(io) => {
                let dur = Duration::from_secs_f32(io);
                &self.player.try_seek(io);


            }
            PlaybackStatus::SeekId(id) => {

            }
            PlaybackStatus::SeekCur(io) => {

            }
            PlaybackStatus::Stop => {


            }




        }



    }

    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
    pub fn playlistinfo(&self) -> MpdSong
    

}
impl NaviApiParse for CurrentSong { 
    fn get_length(&self) -> String {


    }
    fn get_url(&self) -> String {

    }



}
