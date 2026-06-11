use std::time::Duration;
use std::sync::{Arc, Mutex};
use bytes::Bytes;


use reqwest::header::ALLOW;
use reqwest::Client;
use crate::rodio_stub::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, source::Source};
use crate::rodio_stub::Player as RodioPlayer;
use std::io::Cursor;
use crate::MpdSong;
use crate::tracklist::Song;

const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    song_id: Arc<Mutex<String>>,
    bytes: Bytes,
    var: MixerDeviceSink, // Player depends on Mixer (it says on rust document dont forget this)
    queue: PlaybackQueue,
}
pub struct PlaybackQueue {
    items: Vec<Song>,
    cursor: i32,
    player: Player,
}


pub enum PlaybackStatus {
    Seek(f32),
    Next(),
    Pause(i32),
    Play,              // resume / play with no arg
    PlayPos(usize),    // play [songpos]
    PlayId(String),
    Previous,
    SeekId(String),
    SeekCur(f32),
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

pub type SharedState = Arc<tokio::sync::RwLock<PlayerState>>;


impl CurrentSong {
    pub async fn new(song_id: &str, client: Client) -> Self {
        let le_url: String = CurrentSong::fmt_url(song_id);
        let bytes = client
            .get(&le_url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await.unwrap();
        let handle = DeviceSinkBuilder::open_default_sink().unwrap();
        //let plr = rodio::Player::connect_new(&handle.mixer());
        Self {
            song_id: Arc::new(Mutex::new(song_id.to_string())),
            var: handle.clone(),
            bytes: bytes,
            //player: plr,
            queue: PlaybackQueue {
                items: Vec::new(),
                cursor: 0,
                player: Player::connect_new(&handle.mixer()),
            }
        }

    }
    pub fn handle(&mut self,command: PlaybackStatus)  {
        match command {
            PlaybackStatus::Next() => {
                println!("dbg... we're going to the next song...");
                self.queue.next();
            }
            PlaybackStatus::Pause(io) => {
                println!("pause");
                match io {
                    0 => {
                        &self.queue.player.pause();
                    }
                    1 => {
                        &self.queue.player.play();
                    }
                    _ => {
                        println!("unexpected args..");
                    }
                }
            }
            PlaybackStatus::Play => {
                println!("play (resume)");
                self.queue.player.play();
            }
            PlaybackStatus::PlayPos(pos) => {
                println!("play pos {}", pos);
                self.queue.jump_to(pos);
            }
            #[allow(unused_variables)]
            PlaybackStatus::PlayId(io) => {
                println!("play by id");
                for idx in 0..self.queue.items.len() {
                    match &self.queue.items.get(idx) {
                        io => {
                            println!("match id");
                            for itr in 0..idx {
                                println!("{itr}");
                                &self.queue.player.skip_one();
                            }
                        }
                    }
                }



            }
            PlaybackStatus::Previous => {
                println!("previous");
                self.queue.previous();
            }
            PlaybackStatus::Seek(io) => {
                let dur = Duration::from_secs_f32(io);
                &self.queue.player.try_seek(dur);


            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekId(id) => {
                for idx in 0..self.queue.items.len() {
                    match &self.queue.items.get(idx) {
                        id => for i in 0..self.queue.items.len() {
                            println!("found id");
                            &self.queue.jump_to(0);

                        }



                    }

                }


            }
            PlaybackStatus::SeekCur(io) => {
                let var = self.queue.player.get_pos().clone();
                let delta = Duration::from_secs_f32(io);
                for idx in 0..self.queue.items.len() {
                    match self.queue.items.get(idx) {
                        id => for i in 0..self.queue.items.len() {
                            println!("found id");
                            self.queue.jump_to(0);
                            self.queue.player.try_seek(var + delta);
                        }



                    }

                }



            }
            PlaybackStatus::Stop => {
                println!("stop!");
                &self.queue.player.stop();

            }




        }



    }
    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
    pub fn get_queue_cursor(&self) -> i32 {
        self.queue.cursor
    }
    pub fn get_queue_len(&self) -> usize {
        self.queue.items.len()
    }

}
#[allow(unused_variables)]
impl PlaybackQueue {

   pub fn next(&mut self) {
       if self.cursor < self.items.len() as i32 - 1 {
           self.cursor += 1;
           self.player.skip_one();
       }
   }

   pub fn previous(&mut self) {
       self.cursor = self.cursor.saturating_sub(1);
       self.rebuild();
   }

   pub fn remove(&mut self, index: usize) {
       self.items.remove(index);
       if (index as i32) <= self.cursor {
           self.cursor = self.cursor.saturating_sub(1);
       }
       self.rebuild();
   }

   pub fn jump_to(&mut self, index: usize) {
       self.cursor = index as i32;
       self.rebuild();
   }

   fn rebuild(&mut self) {
       self.player.stop();
       for song in &self.items[self.cursor as usize..] {
          // self.player.append();
       }
   }

   fn play_current(&mut self) {
       if let Some(song) = self.items.get(self.cursor as usize) {
         //self.player.append(song_to_source(song));
       }
   }
}
