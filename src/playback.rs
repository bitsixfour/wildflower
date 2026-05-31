use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, source::Source};
use rodio::Player;
use std::io::Cursor;
const URL: &str = "192.168.1.20:8097";

pub struct CurrentSong {
    song_id: Arc<Mutex<String>>,
    stream: Cursor<Bytes>, 
    audo_sink: MixerDeviceSink,
}
pub struct Queue {
    items: Vec<Song>,
    cursor: i32,
}
impl Queue {
    fn current(&self) -> Option<&SongData> { self.items.get(self.cursor).unwrap() }
    fn next(&mut self) -> Option<&SongData> { self.cursor += 1; self.current() }
    fn prev(&mut self) -> Option<&SongData> { self.cursor = self.cursor.saturating_sub(1); self.current() }
    fn add(&mut self, song: SongData) { self.items.push(song); }
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
        let sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        Self {
            song_id: Arc::new(Mutex::new(format!(song_id))),
            stream: bytes,
            audo_sink: sink_handle,

        }

    }
    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
    










}
