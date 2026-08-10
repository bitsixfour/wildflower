use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NavidromeConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

impl NavidromeConfig {
    pub fn from_env() -> Self {
        Self {
            url: Self::env_string("NAVIDROME_URL")
                .or_else(|| Self::env_string("MPD_HOST"))
                .unwrap_or_else(|| "http://127.0.0.1:4533".to_string()),
            username: Self::env_string("NAVIDROME_USER")
                .or_else(|| Self::env_string("MPD_USER"))
                .unwrap_or_else(|| "navidrome".to_string()),
            password: Self::env_string("NAVIDROME_PASSWORD")
                .or_else(|| Self::env_string("MPD_PASS"))
                .unwrap_or_default(),
        }
    }

    pub fn endpoint(&self, resource: &str) -> String {
        format!("{}/rest/{resource}", self.url.trim_end_matches('/'))
    }

    fn env_string(key: &str) -> Option<String> {
        env::var(key).ok().filter(|value| !value.trim().is_empty())
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub navidrome: NavidromeConfig,
    pub mpd_port: u16,
    pub database_path: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            navidrome: NavidromeConfig::from_env(),
            mpd_port: Self::env_u16("MPD_PORT").unwrap_or(6600),
            database_path: Self::env_string("WILDFLOWER_DATABASE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("albumdata.db")),
        }
    }

    fn env_string(key: &str) -> Option<String> {
        env::var(key).ok().filter(|value| !value.trim().is_empty())
    }

    fn env_u16(key: &str) -> Option<u16> {
        Self::env_string(key).and_then(|value| value.parse::<u16>().ok())
    }
}
