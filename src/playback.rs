use std::time::Duration;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use std::collections::VecDeque;



// use reqwest::header::ALLOW;
use reqwest::Client;
use std::io::Cursor;
use rodio::{Decoder, OutputStream, Sink, source::Source};
use std::io::BufReader;
use crate::MpdSong;
use crate::tracklist::Song;

const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    song_id: String,
    stream: Bytes,
    var: MixerDeviceSink, // Player depends on Mixer (it says on rust document dont forget this)
    queue: PlaybackQueue,
}
pub struct PlaybackQueue {
    items: VecDeque<Song>,
    cursor: i32,
    player: Player,
}


pub enum PlaybackStatus {
    Seek((u64, String)),
    Next(),
    Pause(i32),
    Play,              // resume / play with no arg
    PlayPos(usize),    // play [songpos]
    PlayId(String),
    Previous,
    SeekId((u64, String)),
    SeekCur(u64),
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
        Self {
            song_id: le_url,
            var: handle.clone(),
            stream: bytes,
            queue: PlaybackQueue {
                items: VecDeque::new(),
                cursor: 0,
                player: rodio::Sink::connect_new(&handle.mixer()),
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
                self.queue.player.play();
            }
            PlaybackStatus::PlayPos(pos) => {
                println!("play pos {}", pos);
                self.queue.jump_to(pos);
                self.queue.play_current();
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
                                self.queue.player.skip_one();
                            }
                        }
                    }
                }
                self.queue.play_current();



            }
            PlaybackStatus::Previous => {
                println!("previous");
                self.queue.previous();
            }
            PlaybackStatus::Seek(io) => {
                let pos_seek = Duration::from_secs(io.0);
                self.queue.jump_to(io.0 as usize);
                self.queue.player.try_seek(pos_seek);

            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekId(id) => {
                let sec_seek = Duration::from_secs(id.0); 
                for idx in 0..self.queue.items.len() {
                    match self.queue.items.get(idx) {
                        id => for i in 0..self.queue.items.len() {
                            println!("found id");
                            self.queue.jump_to(i);
                            self.queue.player.try_seek(sec_seek);

                        }

                    }

                }


            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekCur(io) => {
                let var = self.queue.player.get_pos().clone();
                let delta = Duration::from_secs(io);
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
                self.queue.player.stop();

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
/* sink such that the actual rodio crate s.t. it's really only 1 active source, but we hide this
 * by just using a vec which we display out using the mpd spec...
 */

#[allow(unused_variables)]
impl PlaybackQueue {
    j


    pub fn next(&mut self) {
        self.items.pop_front();
        




    }

    pub fn previous(&mut self) {
    }


    pub fn remove(&mut self, index: usize) {
    }

    pub fn jump_to(&mut self, index: usize) {
    }




    /* heckin backend functions */

    fn sink_current(&mut self, sink: Sink) {

      
    }
    fn sink_backward(&mut self, sink: Sink) {
        


    }
    fn sink_init(&mut self, sink: &Sink, stream: Vec<u8> ) {
        let cursor = Cursor::new(stream);
        let source = Decoder::new(cursor);
        sink.append(source);
        sink.play();
    }
    /* buffer up to two at a time for "gapless" playback*/
    async fn rebuild_buffer(&mut self, sink: &Sink, client: &Client) {
        match (self.items.front(), self.items.get(1)) {
            (Some(x), Some(y)) => {
                let str = Self::get_audio_stream(client, &x.id);
                let str_2 = Self::get_audio_stream(client, &x.id);
                let source_1 = Decoder::new(Cursor::new(str));
                let source_2 = Decoder::new(Cursor::new(str_2));
                self.player.append(source_1);
                self.player.append(source_2);



            }
            (Some(x), None) => {
                let str = Self::get_audio_stream(client, &x.id);
                let source = Decoder::new(Cursor::new(str));
                self.player.append(source);


            }
            _ => {
                println!("msg: no need to buffer");


            }




        }





    }
    // todo: hide in interface later
    async fn get_audio_stream(client: &Client, search_id: &str) -> Vec<u8> {
        let req = format!(STR, search_id);
        let mut vec: Vec<u8> = Vec::new();
        let mut bytes: Vec<u8> = reqwest::Client::new()
            .get(req)
            .send().await?
            .error_for_status()?
            .bytes().await?
            .to_vec();
        vec.append(&mut bytes);


        vec


        

        }
        

}
