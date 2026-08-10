use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::config::NavidromeConfig;
use crate::play::tracklist::{Song, SubsIDResponse};

const SQL_SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS albums (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    artist          TEXT NOT NULL,
    artist_id       TEXT NOT NULL,
    cover_art       TEXT NOT NULL,
    song_count      INTEGER NOT NULL,
    duration        INTEGER NOT NULL,
    created         TEXT NOT NULL,
    year            INTEGER,
    genre           TEXT,
    user_rating     INTEGER,
    music_brainz_id TEXT,
    is_compilation  INTEGER NOT NULL DEFAULT 0,
    sort_name       TEXT,
    raw_json        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS genres (
    album_id TEXT NOT NULL,
    name     TEXT NOT NULL,
    PRIMARY KEY (album_id, name),
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS songs (
    id                   TEXT PRIMARY KEY,
    album_id             TEXT NOT NULL,
    parent               TEXT NOT NULL,
    is_dir               INTEGER NOT NULL DEFAULT 0,
    title                TEXT NOT NULL,
    album                TEXT NOT NULL,
    artist               TEXT NOT NULL,
    track                INTEGER NOT NULL,
    year                 INTEGER NOT NULL,
    cover_art            TEXT NOT NULL,
    file_size            INTEGER NOT NULL,
    content_type         TEXT NOT NULL,
    suffix               TEXT NOT NULL,
    duration             INTEGER NOT NULL,
    bit_rate             INTEGER NOT NULL,
    path                 TEXT NOT NULL,
    play_count           INTEGER,
    created              TEXT NOT NULL,
    artist_id            TEXT NOT NULL,
    media_type           TEXT NOT NULL,
    played               TEXT,
    bpm                  INTEGER NOT NULL,
    comment              TEXT NOT NULL,
    sort_name            TEXT NOT NULL,
    media_type_tag       TEXT NOT NULL,
    channel_count        INTEGER NOT NULL,
    sampling_rate        INTEGER NOT NULL,
    bit_depth            INTEGER NOT NULL,
    display_artist       TEXT NOT NULL,
    display_album_artist TEXT NOT NULL,
    raw_json             TEXT NOT NULL,
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS album_songs (
    album_id TEXT NOT NULL,
    song_id  TEXT NOT NULL,
    PRIMARY KEY (album_id, song_id),
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS songs_album_id ON songs(album_id);
CREATE INDEX IF NOT EXISTS songs_path ON songs(path);
CREATE INDEX IF NOT EXISTS songs_title ON songs(title);
CREATE INDEX IF NOT EXISTS albums_artist ON albums(artist);
CREATE INDEX IF NOT EXISTS albums_name ON albums(name);
"#;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Root {
    #[serde(rename = "subsonic-response")]
    subsonic_response: SubsonicResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubsonicResponse {
    status: String,
    version: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "serverVersion")]
    server_version: String,
    #[serde(rename = "openSubsonic")]
    open_subsonic: bool,
    #[serde(rename = "albumList2")]
    album_list_2: AlbumList2,
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, Clone, PartialEq)]
pub struct AlbumList2 {
    album: Vec<Album>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: String,

    #[serde(rename = "artistId")]
    pub artist_id: String,
    #[serde(rename = "coverArt")]
    pub cover_art: String,
    #[serde(rename = "songCount")]
    pub song_count: u32,
    pub duration: u32,
    pub created: String,
    pub year: Option<u32>,
    pub genre: Option<String>,
    #[serde(rename = "userRating")]
    pub user_rating: Option<u32>,
    pub genres: Vec<Genre>,
    #[serde(rename = "musicBrainzId")]
    pub music_brainz_id: Option<String>,
    #[serde(rename = "isCompilation")]
    pub is_compilation: bool,
    #[serde(rename = "sortName")]
    pub sort_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct Genre {
    name: String,
}

pub fn get_url(config: &NavidromeConfig, song_id: &str) -> String {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("id", song_id)
        .append_pair("u", &config.username)
        .append_pair("p", &config.password)
        .append_pair("v", "1.8.0")
        .append_pair("c", "wildflower")
        .finish();
    format!("{}?{query}", config.endpoint("stream"))
}

pub async fn navi_obj(
    client: &Client,
    config: &NavidromeConfig,
) -> Result<SubsonicResponse, reqwest::Error> {
    let root = client
        .get(config.endpoint("getAlbumList2"))
        .query(&[
            ("u", config.username.as_str()),
            ("p", config.password.as_str()),
            ("v", "1.8.0"),
            ("c", "wildflower"),
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

    Ok(root.subsonic_response)
}

#[derive(Clone)]
pub struct NaviData {
    pub album_list: Vec<Album>,
    pub albums_cache: HashMap<String, Vec<Song>>,
    pub songs_cache: HashMap<String, Vec<Song>>,
    pub sql_connection: Arc<Mutex<Connection>>,
    pub config: NavidromeConfig,
}

impl NaviData {
    pub fn init_empty() -> Self {
        Self::init_empty_with_config(NavidromeConfig::from_env())
    }

    pub fn init_empty_with_config(config: NavidromeConfig) -> Self {
        let connection = Connection::open_in_memory().expect("open in-memory SQLite database");
        initialize_schema(&connection).expect("initialize in-memory SQLite schema");
        Self::from_connection(Arc::new(Mutex::new(connection)), config)
            .expect("load empty in-memory SQLite library")
    }

    pub async fn load_or_fetch(
        client: &Client,
        db_path: &Path,
        config: NavidromeConfig,
    ) -> Result<Self> {
        let connection = Connection::open(db_path)
            .with_context(|| format!("open SQLite database {}", db_path.display()))?;
        initialize_schema(&connection).context("initialize SQLite schema")?;
        let shared = Arc::new(Mutex::new(connection));
        let cached = Self::from_connection(Arc::clone(&shared), config.clone())
            .context("load SQLite library")?;

        let response = match navi_obj(client, &config).await {
            Ok(response) => response,
            Err(_error) if !cached.album_list.is_empty() => return Ok(cached),
            Err(error) => return Err(error).context("getAlbumList2 failed"),
        };

        if let Err(error) =
            refresh_database(&shared, response.album_list_2.album, client, &config).await
        {
            if !cached.album_list.is_empty() {
                return Ok(cached);
            }
            return Err(error).context("refresh SQLite library");
        }

        Self::from_connection(shared, config).context("load refreshed SQLite library")
    }

    pub async fn init_cache(
        data: Vec<Album>,
        client: &Client,
        config: &NavidromeConfig,
    ) -> HashMap<String, Vec<Song>> {
        let mut songs = HashMap::with_capacity(data.len());
        for album in data {
            let album_songs = fetch_album_songs(client, &album.id, config)
                .await
                .unwrap_or_default();
            songs.insert(album.id, album_songs);
        }
        songs
    }

    pub async fn updt(resp: SubsonicResponse, client: &Client, config: NavidromeConfig) -> Self {
        let navi = Self::init_empty_with_config(config.clone());
        refresh_database(
            &navi.sql_connection,
            resp.album_list_2.album,
            client,
            &config,
        )
        .await
        .expect("store Navidrome response in SQLite");
        Self::from_connection(navi.sql_connection, config)
            .expect("load SQLite snapshot after update")
    }
    pub fn album_songs(&self, album_id: &str) -> Vec<Song> {
        self.albums_cache.get(album_id).cloned().unwrap_or_default()
    }

    pub fn song_by_id(&self, song_id: &str) -> Option<Song> {
        self.albums_cache
            .values()
            .flat_map(|songs| songs.iter())
            .find(|song| song.id == song_id)
            .cloned()
    }

    fn from_connection(
        sql_connection: Arc<Mutex<Connection>>,
        config: NavidromeConfig,
    ) -> Result<Self> {
        let connection = sql_connection
            .lock()
            .map_err(|_| anyhow!("SQLite connection mutex poisoned"))?;

        let mut albums_statement =
            connection.prepare("SELECT raw_json FROM albums ORDER BY rowid")?;
        let album_json: Vec<String> = albums_statement
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        drop(albums_statement);

        let album_list: Vec<Album> = album_json
            .into_iter()
            .map(|json| serde_json::from_str(&json).context("decode album JSON from SQLite"))
            .collect::<Result<_>>()?;

        let mut songs_statement =
            connection.prepare("SELECT album_id, raw_json FROM songs ORDER BY rowid")?;
        let song_rows: Vec<(String, String)> = songs_statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;

        let mut albums_cache: HashMap<String, Vec<Song>> = HashMap::new();
        for (album_id, json) in song_rows {
            let song: Song = serde_json::from_str(&json).context("decode song JSON from SQLite")?;
            albums_cache.entry(album_id).or_default().push(song);
        }

        let mut songs_cache = albums_cache.clone();
        for album in &album_list {
            if let Some(songs) = albums_cache.get(&album.id) {
                songs_cache.insert(album.name.clone(), songs.clone());
            }
        }

        drop(songs_statement);
        drop(connection);
        Ok(Self {
            album_list,
            albums_cache,
            songs_cache,
            sql_connection,
            config,
        })
    }
}

async fn fetch_album_songs(
    client: &Client,
    album_id: &str,
    config: &NavidromeConfig,
) -> Result<Vec<Song>> {
    let response = client
        .get(config.endpoint("getAlbum"))
        .query(&[
            ("id", album_id),
            ("u", config.username.as_str()),
            ("p", config.password.as_str()),
            ("v", "1.8.0"),
            ("c", "wildflower"),
            ("f", "json"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<SubsIDResponse>()
        .await?;
    Ok(response.subsonic_response.album.song)
}

async fn refresh_database(
    sql_connection: &Arc<Mutex<Connection>>,
    albums: Vec<Album>,
    client: &Client,
    config: &NavidromeConfig,
) -> Result<()> {
    let mut snapshot = Vec::with_capacity(albums.len());
    for album in albums {
        let songs = fetch_album_songs(client, &album.id, config)
            .await
            .with_context(|| format!("fetch songs for album {}", album.id))?;
        snapshot.push((album, songs));
    }

    persist_snapshot(sql_connection, &snapshot)
}

fn persist_snapshot(
    sql_connection: &Arc<Mutex<Connection>>,
    snapshot: &[(Album, Vec<Song>)],
) -> Result<()> {
    let mut connection = sql_connection
        .lock()
        .map_err(|_| anyhow!("SQLite connection mutex poisoned"))?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "DELETE FROM album_songs; DELETE FROM songs; DELETE FROM genres; DELETE FROM albums;",
    )?;

    for (album, songs) in snapshot {
        let album_json = serde_json::to_string(album).context("encode album JSON")?;
        transaction.execute(
            "INSERT INTO albums
             (id, name, artist, artist_id, cover_art, song_count, duration, created,
              year, genre, user_rating, music_brainz_id, is_compilation, sort_name, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                album.id,
                album.name,
                album.artist,
                album.artist_id,
                album.cover_art,
                i64::from(album.song_count),
                i64::from(album.duration),
                album.created,
                album.year.map(i64::from),
                album.genre,
                album.user_rating.map(i64::from),
                album.music_brainz_id,
                i64::from(album.is_compilation as u8),
                album.sort_name,
                album_json,
            ],
        )?;

        for genre in &album.genres {
            transaction.execute(
                "INSERT INTO genres (album_id, name) VALUES (?1, ?2)",
                params![album.id, genre.name],
            )?;
        }

        for song in songs {
            let song_json = serde_json::to_string(song).context("encode song JSON")?;
            transaction.execute(
                "INSERT INTO songs
                 (id, album_id, parent, is_dir, title, album, artist, track, year, cover_art,
                  file_size, content_type, suffix, duration, bit_rate, path, play_count, created,
                  artist_id, media_type, played, bpm, comment, sort_name, media_type_tag,
                  channel_count, sampling_rate, bit_depth, display_artist, display_album_artist, raw_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28,
                         ?29, ?30, ?31)",
                params![
                    song.id,
                    album.id,
                    song.parent,
                    i64::from(song.is_dir as u8),
                    song.title,
                    song.album,
                    song.artist,
                    i64::from(song.track),
                    i64::from(song.year),
                    song.cover_art,
                    i64::try_from(song.size).unwrap_or(i64::MAX),
                    song.content_type,
                    song.suffix,
                    i64::from(song.duration),
                    i64::from(song.bit_rate),
                    song.path,
                    song.play_count.map(i64::from),
                    song.created,
                    song.artist_id,
                    song.media_type,
                    song.played,
                    i64::from(song.bpm),
                    song.comment,
                    song.sort_name,
                    song.media_type_tag,
                    i64::from(song.channel_count),
                    i64::from(song.sampling_rate),
                    i64::from(song.bit_depth),
                    song.display_artist,
                    song.display_album_artist,
                    song_json,
                ],
            )?;
            transaction.execute(
                "INSERT INTO album_songs (album_id, song_id) VALUES (?1, ?2)",
                params![album.id, song.id],
            )?;
        }
    }

    transaction.commit()?;
    Ok(())
}

fn initialize_schema(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(SQL_SCHEMA)
        .context("create SQLite schema")
}
