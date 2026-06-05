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
        //let plr = rodio::Player::connect_new(&handle.mixer());
        Self {
            song_id: Arc::new(Mutex::new(format!(song_id))),
            var: handle,
            //player: plr,
            queue: PlaybackQueue {
                items: Vec::new(),
                cursor: 0,
                player: rodio::Player::connect_new(&handle.mixer()),
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
            PlaybackStatus::Play(io) => {
                println!("play");
                let delta = Duration::as_secs(&io - &self.queue.player.len());
                for _ in 0..delta { 
                    &self.queue.player.skip_one();
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
/*
    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
*/  

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
