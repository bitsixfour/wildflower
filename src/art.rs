use reqwest::Client;
use crate::NaviData;
use crate::database::DatabaseStatus;
const URL: &str = "http://192.168.1.20";


pub async fn return_album_art(req: &str, return_offset: i64) -> Vec<u8> {
    let url: String = format!("{}/rest/getCoverArt?id={}&u=nix&p=2008&v=1.16.1&c=test", URL, req);
    let response = reqwest::get(&url).await.unwrap();
    let all_bytes = response.bytes().await.unwrap();
    
    let offset = return_offset as usize;
    let chunk = if offset < all_bytes.len() {
        all_bytes[offset..].to_vec()
    } else {
        Vec::new()
    };
    
    let mut out = Vec::new();
    out.extend_from_slice(format!("size: {}\nbinary: {}\n", all_bytes.len(), chunk.len()).as_bytes());
    out.extend_from_slice(&chunk);
    out.push(b'\n');
    out.extend_from_slice(b"OK\n");
    out

}

