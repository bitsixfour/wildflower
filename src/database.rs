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



pub fn database_handle(command: DatabaseStatus, client: &Client) {
    match command {
        AlbumArt(id, ost) => {


        }
        _ => println!("unexpected args?")




    }



}
