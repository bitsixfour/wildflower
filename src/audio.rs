
use crate::navi::{NaviData, SubsonicResponse};
use crate::tracklist::Song;
use crate::playback::{CurrentSong, PlaybackStatus, PlayerState, AudioState, SharedState};
use reqwest::Client;
use std::io::Cursor;
const str: &str = "http://192.168.1.20:8097/rest/getAlbum?id={}&u=nix&p=2008&v=1.8.0&c=myapp&f=json"



/* interface to get audio */
mod audio_buffer {


    async fn get_audio(client: &Client, album_data: &NaviData, search: &str) {
         fj





    }






}
