use crate::navidrome::navi::Album;
use crate::play::tracklist::Song;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldOp {
    Contains,
    Eq,
    NotEq,
}

#[derive(Debug, Clone)]
pub enum Expr {
    And(Box<Field>, Box<Field>),
    Or(Box<Field>, Box<Field>),
    Def(Field),
    Empty,
}
#[derive(Debug, Clone)]
pub enum HandleDatabase {
    AlbumArt(String, i32),
    Count(String),
    GetFinger(String),
    LsInfo(String),
    LFiles(String),
    ReadComments(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub field: String,
    pub op: FieldOp,
    pub value: String,
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
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
        match self.op {
            FieldOp::Contains => {
                let val = match self.field.to_lowercase().as_str() {
                    "name" => &album.name,
                    "artist" => &album.artist,
                    "genre" => album.genre.as_deref().unwrap_or(""),
                    _ => return eq,
                };
                contains_ci(val, &self.value)
            }
            FieldOp::Eq => eq,
            FieldOp::NotEq => !eq,
        }
    }

    pub fn matches_song(&self, song: &Song) -> bool {
        let eq = match self.field.to_lowercase().as_str() {
            "title" => song.title == self.value,
            "artist" => song.artist == self.value,
            "album" => song.album == self.value,
            "year" | "date" => song.year.to_string() == self.value,
            "track" => song.track.to_string() == self.value,
            "id" => song.id == self.value,
            _ => false,
        };
        match self.op {
            FieldOp::Contains => {
                let val = match self.field.to_lowercase().as_str() {
                    "title" => &song.title,
                    "artist" => &song.artist,
                    "album" => &song.album,
                    _ => return eq,
                };
                contains_ci(val, &self.value)
            }
            FieldOp::Eq => eq,
            FieldOp::NotEq => !eq,
        }
    }

    pub fn group_value(&self, album: &Album) -> String {
        match self.field.to_lowercase().as_str() {
            "name" | "album" => album.name.clone(),
            "artist" => album.artist.clone(),
            "genre" => album.genre.clone().unwrap_or_default(),
            "year" | "date" => album.year.map(|y| y.to_string()).unwrap_or_default(),
            _ => String::new(),
        }
    }

    pub fn song_group_value(&self, song: &Song) -> String {
        match self.field.to_lowercase().as_str() {
            "title" => song.title.clone(),
            "artist" => song.artist.clone(),
            "album" => song.album.clone(),
            "year" | "date" => song.year.to_string(),
            "track" => song.track.to_string(),
            _ => String::new(),
        }
    }
}

impl Expr {
    pub fn matches_album(&self, album: &Album) -> bool {
        match self {
            Expr::And(a, b) => a.matches(album) && b.matches(album),
            Expr::Or(a, b) => a.matches(album) || b.matches(album),
            Expr::Def(f) => f.matches(album),
            Expr::Empty => true,
        }
    }

    pub fn matches_song(&self, song: &Song) -> bool {
        match self {
            Expr::And(a, b) => a.matches_song(song) && b.matches_song(song),
            Expr::Or(a, b) => a.matches_song(song) || b.matches_song(song),
            Expr::Def(f) => f.matches_song(song),
            Expr::Empty => true,
        }
    }

    pub fn song_group_value(&self, song: &Song) -> String {
        match self {
            Expr::Def(f) => f.song_group_value(song),
            Expr::And(f, _) => f.song_group_value(song),
            Expr::Or(f, _) => f.song_group_value(song),
            Expr::Empty => String::new(),
        }
    }
}
