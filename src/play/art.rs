use reqwest::Client;

use crate::config::NavidromeConfig;

pub async fn return_album_art(
    req: &str,
    return_offset: i64,
    client: &Client,
    config: &NavidromeConfig,
) -> Vec<u8> {
    let response = match client
        .get(config.endpoint("getCoverArt"))
        .query(&[
            ("id", req),
            ("u", config.username.as_str()),
            ("p", config.password.as_str()),
            ("v", "1.16.1"),
            ("c", "wildflower"),
        ])
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            eprintln!("album art request failed for {req}: {error}");
            return b"ACK [50@0] {albumart} request failed\n".to_vec();
        }
    };

    let all_bytes = match response.error_for_status() {
        Ok(response) => match response.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("album art response failed for {req}: {error}");
                return b"ACK [50@0] {albumart} response failed\n".to_vec();
            }
        },
        Err(error) => {
            eprintln!("album art request returned an error for {req}: {error}");
            return b"ACK [50@0] {albumart} request failed\n".to_vec();
        }
    };

    let offset = return_offset.max(0) as usize;
    let chunk = if offset < all_bytes.len() {
        &all_bytes[offset..]
    } else {
        &[]
    };

    let mut out = Vec::with_capacity(chunk.len() + 64);
    out.extend_from_slice(
        format!("size: {}\nbinary: {}\n", all_bytes.len(), chunk.len()).as_bytes(),
    );
    out.extend_from_slice(chunk);
    out.extend_from_slice(b"\nOK\n");
    out
}
