use crate::MpdSong;
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



pub async fn database_handle(command: DatabaseStatus, client: &Client, navi: NaviData) -> String {
    match command {
        AlbumArt(id, ost) => {
            let resp = art::return_album_art(id, ost).await;
            resp

        }
        Count(filter_str, group_type) => {
            let mut parts = filter_str.splitn(2, ' ');
            let field = parts.next().unwrap_or("").to_lowercase();
            let value = parts.next().unwrap_or("").to_string();

            let mut songs: u32 = 0;
            let mut playtime: u32 = 0;

            for album in &navi.album_list {
                let hit = match field.as_str() {
                    "title" | "album" => album.name.to_lowercase().contains(&value.to_lowercase()),
                    "artist" => album.artist.to_lowercase().contains(&value.to_lowercase()),
                    "genre" => album.genre.as_deref().unwrap_or("").to_lowercase().contains(&value.to_lowercase()),
                    "date" | "year" => album.year.map_or(false, |y| y.to_string() == value),
                    _ => false,
                };
                if hit {
                    songs += album.song_count;
                    playtime += album.duration;
                }
            }

            let mut out = format!("songs: {}\nplaytime: {}\n", songs, playtime);
            if !group_type.is_empty() {
                out.push_str(&format!("group: {}\n", group_type));
            }
            out.push_str("OK\n");
            out
        }

        _ => format!("ACK-!")




    }



}
