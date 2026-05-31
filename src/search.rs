/* This is so that we can use the Enum Expr
 * and Field so that we can get the ID so then it would
 * PROBABLY be possible to use search.rs to return the fat
 * thingy of data (the full tracklist info)
 */

use crate::navi::Album;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    And(Field, Field),
    Or(Field, Field),
    Def(Field),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub field: String,
    pub op: bool,
    pub value: String,
}

impl Field {
    pub fn matches(&self, album: &Album) -> bool {
        let eq = match self.field.as_str() {
            "id" | "Id"               => album.id == self.value,
            "name" | "Name"           => album.name == self.value,
            "artist" | "Artist"       => album.artist == self.value,
            "artistId" | "artist_id"  => album.artist_id == self.value,
            "coverArt" | "cover_art"  => album.cover_art == self.value,
            "songCount" | "song_count" => album.song_count.to_string() == self.value,
            "duration" | "Duration"   => album.duration.to_string() == self.value,
            "created" | "Created"     => album.created == self.value,
            "year" | "Year"           => album.year.map_or(false, |y| y.to_string() == self.value),
            "genre" | "Genre"         => album.genre.as_deref() == Some(&self.value),
            "userRating" | "user_rating" => album.user_rating.map_or(false, |r| r.to_string() == self.value),
            "musicBrainzId" | "music_brainz_id" => album.music_brainz_id.as_deref() == Some(&self.value),
            "isCompilation" | "is_compilation" => album.is_compilation.to_string() == self.value,
            "sortName" | "sort_name"  => album.sort_name.as_deref() == Some(&self.value),
            _ => false,
        };

        if self.op {
            eq
        } else {
            !eq
        }
    }
}

impl Expr {
    pub fn create_new(&self, album: &Album) -> bool {
        match self {
            Expr::And(a, b) => a.matches(album) && b.matches(album),
            Expr::Or(a, b) => a.matches(album) || b.matches(album),
            Expr::Def(f) => f.matches(album),
            Expr::Empty => true,
        }
    }
}
