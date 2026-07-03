use std::env;

pub struct MpdCfg {
    pub url: String,
    pub usr: String,
    pub pass: String,
    pub port: u32,
}

impl MpdCfg {
    pub fn from_env() -> Self {
        Self {
            usr: Self::env_string("MPD_USER").unwrap_or_else(|| "mpd".to_string()),
            pass: Self::env_string("MPD_PASS").unwrap_or_else(|| String::new()),
            port: Self::env_u32("MPD_PORT").unwrap_or(6600),
            url: Self::env_string("MPD_HOST").unwrap_or_else(|| "localhost".to_string()),
        }
    }

    fn env_string(key: &str) -> Option<String> {
        env::var(key).ok().filter(|value| !value.trim().is_empty())
    }

    fn env_u32(key: &str) -> Option<u32> {
        Self::env_string(key).and_then(|value| value.parse::<u32>().ok())
    }
}
