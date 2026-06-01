use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, source::Source};
use rodio::Player;
use std::io::Cursor;
const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    song_id: Arc<Mutex<String>>,
    stream: Cursor<Bytes>, 
    var: MixerDeviceSink, // Player depends on Mixer (it says on rust document dont forget this)
    player: Player,
}
pub struct Queue {
    items: Vec<Song>,
    cursor: i32,
}
pub enum PlaybackStatus {
    Statu1
    



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
            stream: bytes,
            var: handle,
            player: plr,
        }

    }
    pub fn handle(&self, enum: PlaybackStatus) -> anyhow::Result<String> {



    }

    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
    

}
impl NaviApiParse for CurrentSong { 
    fn get_length(&self) -> String {


    }
    fn get_url(&self) -> String {

    }



}
