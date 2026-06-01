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
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let plr = rodio::Player::connect_new(&handle.mixer());
        Self {
            song_id: Arc::new(Mutex::new(format!(song_id))),
            stream: bytes,
            var: handle,
            player: plr,
            

        }

    }
    // play [SONGPOS] (mpd documentation says you have to get an input arg)
    pub fn rodio_play(&self, x: u32)  {
        let pos = Duration::from_secs_f32(x);
        &self.player.try_seek(pos);
    }
    /* rodio library already implements most of these. note: dont make retarded wrapper 
     * classes for these
    pub fn rodio_stop(&self) => bool {
        



    }
    */
    pub fn rodio_seek(&self, pos: u8)  {
        let pos = Duration::from_secs_f32(pos);
        &self.player.try_seek(pos);

        


    }

    pub fn fmt_url(io: &str) -> String {
        let endpnt = format!("http://{}/rest/stream?u=nix&p=2008&v=1.16.1&c=test&id={}",
            URL,
            io);
        endpnt
    }
    










}
