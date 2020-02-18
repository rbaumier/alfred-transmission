use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Torrent {
    pub id: i32,
    pub name: String,
    status: i32,
    is_finished: bool,
    pub rate_download: u128,
    pub percent_done: f32,
    pub download_dir: String,
    #[serde(with = "ts_seconds")]
    pub added_date: DateTime<Utc>,
    #[serde(with = "seconds_duration")]
    pub eta: Duration,
}

#[derive(Debug)]
pub enum TorrentStatus {
    Active,
    Paused,
    Finished,
}

impl fmt::Display for TorrentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Torrent {
    pub fn status(&self) -> TorrentStatus {
        use TorrentStatus::*;
        if self.is_finished() {
            Finished
        } else if self.is_paused() {
            Paused
        } else {
            Active
        }
    }

    fn is_paused(&self) -> bool {
        self.status == 0 && !self.is_finished
    }

    fn is_finished(&self) -> bool {
        self.percent_done == 1.0
    }
}

mod seconds_duration {
    use chrono::Duration;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        Ok(Duration::seconds(secs))
    }
}
