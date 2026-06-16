use std::time::Duration;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use std::collections::VecDeque;



// use reqwest::header::ALLOW;
use reqwest::Client;
use std::io::Cursor;
use rodio::{Decoder, Player, MixerDeviceSink};
use std::io::BufReader;
use crate::MpdSong;
use crate::tracklist::Song;

const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    pub song_id: String,
    pub stream: Bytes,
    var: MixerDeviceSink, // Player depends on Mixer (it says on rust document dont forget this)
    pub queue: PlaybackQueue,
}
pub struct PlaybackQueue {
    pub items: Vec<Song>,
    pub cursor: i32,
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

/* Move these two args 
 * somewhere else when all core functions are satisfied */
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
/*
pub enum DatabaseStatus {
    AlbumArt(String, i64),
    Count(String, String),
    //GetFinderPrint2(String),
    Find(String, String),
    FindAdd(Vec<&str>),
    Lis(Vec<&str>),
    ListAll(Box<&str>),
    ListAllInfo(Box<&str>),
    ListFiles(&str),
    LsInfo(&str),
    ReadComment(&str),
    ReadPicture(&str),
    SearchAdd(Vec<&str>),
    Searchaddpi(Vec<&str>),
    SearchCount(Vec<&str>),
    Update(),
    Rescan()
}
*/



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
    pub async fn new(song_id: &str, client: &Client) -> Self {
        let le_url: String = CurrentSong::fmt_url(song_id);
        let bytes = client
            .get(&le_url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await.unwrap();
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let mixer = handle.mixer().clone();
        Self {
            song_id: le_url,
            var: handle,
            stream: bytes,
            queue: PlaybackQueue {
                items: Vec::new(),
                cursor: 0,
                player: Player::connect_new(&mixer),
            }
        }

    }
    pub async fn handle(&mut self,command: PlaybackStatus, client: &Client)  {
        match command {
            PlaybackStatus::Next() => {
                println!("dbg... we're going to the next song...");
                self.queue.next(client);
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
                self.queue.rebuild_buffer(client);
                self.queue.player.play();
            }
            PlaybackStatus::PlayPos(pos) => {
                println!("play pos {}", pos);
                let dur = Duration::from_secs(pos as u64);
                self.queue.player.try_seek(dur);

            }
            #[allow(unused_variables)]
            PlaybackStatus::PlayId(io) => {
                println!("play by id");
                self.queue.player.stop();
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
                self.queue.player.play();
            }
            PlaybackStatus::Previous => {
                println!("previous");
                self.queue.previous(client);
            }
            PlaybackStatus::Seek(io) => {
                let pos_seek = io.0.clone();
                // self.queue.jump_to(client, pos_seek);
                self.queue.jump_to(client, pos_seek as i32);
            }
            #[allow(unused_variables)]
            PlaybackStatus::SeekId(id) => {
                let sec_seek = Duration::from_secs(id.0); 
                for idx in 0..self.queue.items.len() {
                    match self.queue.items.get(idx) {
                        id => for i in 0..self.queue.items.len() {
                            println!("found id");
                            self.queue.jump_to(client, i as i32);
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
                            self.queue.jump_to(client, i as i32);
                            self.queue.player.try_seek(var + delta);
                        }
                    }
                }
            }
            PlaybackStatus::Stop => {
                println!("stop!");
                self.queue.player.stop();

            }
            // move later 




        }



    }




    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }

}
/* sink such that the actual rodio crate s.t. it's really only 1 active source, but we hide this
 * by just using a vec which we display out using the mpd spec...
 */

#[allow(unused_variables)]
impl PlaybackQueue {

    pub fn next(&mut self, client: &Client) {
        self.cursor += 1;
        self.player.play();
        self.buffer(client, false);

    }



    pub fn previous(&mut self, client: &Client) {
        self.cursor -= 1;
        self.buffer(client, true);
    
    }


    pub fn remove(&mut self, client: &Client, idx: i32) {
        let rm = idx as usize;
        self.items.remove(rm);
    }

    pub async fn jump_to(&mut self, client: &Client, idx: i32) {
        self.cursor = idx;
        self.rebuild_buffer(client).await;
        
        
    }




    /* heckin backend functions */
    fn sink_init(&mut self, stream: Vec<u8>, client: &Client) {
        let source = Decoder::new(Cursor::new(stream)).unwrap();
        self.player.append(source);
        self.rebuild_buffer(client);
        self.player.play();
    }
    /* true to decrement */
    async fn buffer(&mut self, client: &Client, backwards: bool) {
        let mut var: i32 = 1;
        if backwards {
            var = var - 2;
        }
        let now: i32 = self.cursor.clone() + var;
        let item_id = self.items.get(now as usize).map(|item| item.id.clone()).unwrap();
        match self.items.get(now as usize) {
            Some(now) => {
                let stream = self.get_audio_stream(item_id.as_str(), client).await;
                let buf_next = Decoder::new(Cursor::new(stream));
                self.player.append(buf_next.unwrap());


            }
            _ => {
                println!("nothing left in queue.. no buff");

            }

        }


    }

    async fn destroy_buffer(&mut self) {
        self.player.clear();
    }
    /* queue up to two at a time for "gapless" playback for now"*/
    async fn rebuild_buffer(&mut self, client: &Client) {                   
       let var = self.cursor as usize;                                     
                                                                           
       let id_prev = self.items.get(var - 1).map(|s| s.id.clone());        
       let id_curr = self.items.get(var).map(|s| s.id.clone());            
                                                                           
       match (id_prev, id_curr) {                                          
           (Some(ref x_id), Some(ref y_id)) => {                           
               let str = self.get_audio_stream(x_id, client).await;        
               let str_2 = self.get_audio_stream(y_id, client).await;      
               let source_1 = Decoder::new(Cursor::new(str));              
               let source_2 = Decoder::new(Cursor::new(str_2));            
               self.player.append(source_1.unwrap());                      
               self.player.append(source_2.unwrap());                      
           }                                                               
           (Some(ref x_id), None) => {                                     
               let str = self.get_audio_stream(x_id, client).await;        
               let source = Decoder::new(Cursor::new(str));                
               self.player.append(source.unwrap());                        
           }                                                               
           _ => {                                                          
               println!("msg: no need to buffer");                         
           }                                                               
       }                                                                   
    }
    /* return Result<T> later, but use unwrap now
     * until later */
    async fn get_audio_stream(&mut self, search_id: &str, client: &Client) -> Vec<u8> {
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
