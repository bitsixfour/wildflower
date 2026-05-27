use std::collections::HashMap;
use reqwest::Client;

pub struct FullAlbum {
    full_album: Vec<MpdAlbum>,
}


pub struct MpdAlbum {
    file: &str,
    title: &str,
    artist: &str,
    album: &str,
    duration: f32,
    track: i16,
}


impl FullAlbum {
    async fn new(alb: &NaviData, ser: &str) -> Vec<MpdAlbum> {
        let vec: Vec<MpdAlbum> = Vec::new();
        println!("currentsong");
        let uid: &str = alb.get("ser").unwrap();
        let url = format!("http://192.168.1.20:8097/rest/getAlbum?id={}&u=nix&v=1.8.0&c=myapp", uid);
        let root = client
            .get(url)
            .query(&[
                ("f", "json"),
                ("type", "alphabeticalByName"),
                ("size", "500"),
                ("offset", "0"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Root>()
        .await?;

        return vec
        
    }


}
