pub struct Song {
    file: &str,
    title: &str,
    artist: &str,
    album: &str,
    duration: f32,
    track: i16,
}



impl Song {
    pub async fn get(resp: &NaviData) -> Self {
        println!("finding data");    


    }


}
