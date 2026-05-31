/* Fufill MPD's protocol b/c the docs include a "stats" protocol. 
 * Could make this less LOC by integrating it into something else but idgaf */
pub struct PlayerState {
   // state: do later
   current_song: Option<u16>,
   current_id: Option<i64>,
   elapsed: f64,
   volume: u8,
   repeat: bool,
   random: bool,
   playlist_version: u32,
   playlist: Vec<Song>,

}

impl PlayerState {
    pub fn new_struct() -> PlayerState {
        // need MPD daemon to work b4 doing this. ideally starts at startup


    }
    pub fn upt_struct(&self) -> PlayerState {
        // need MPD daemon to work b4 doing this

    }




}
