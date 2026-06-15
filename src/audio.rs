
use crate::navi::{NaviData, SubsonicResponse};
use crate::tracklist::Song;
use crate::playback::{CurrentSong, PlaybackStatus, PlayerState, AudioState, SharedState};
use reqwest::Client;
use std::io::Cursor;

/* No salt; add later
 */
const STR: &str = "http://192.168.1.20:8097/rest/stream?id={}&u=nix&p=2008&v=1.16.1&c=myapp&f=json&format=raw"




/* interface for audio */
mod audio_buffer {


    async fn get_audio_stream(client: &Client, album_data: &NaviData, search_id: &str) -> Vec<u8> {
        let req = format!(STR, search_id);
        let mut vec: Vec<u8> = Vec::new();
        if let var = album_data.data.contains_key(search_id) {
            let mut bytes: Vec<u8> = reqwest::Client::new()
                .get(req)
                .send().await?
                .error_for_status()?
                .bytes().await?
                .to_vec();
            vec.append(&mut bytes);



        } else {
            println!("could not fetch stream");

        }
        vec


        

        }
        





}




