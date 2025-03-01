use std::fs::File;
use std::io::{BufRead, BufReader};

use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawConfig {
    #[serde(rename = "roomId")]
    pub room_id: u64,
    #[serde(rename = "uid")]
    pub uid: Option<u64>,
    pub sessdata: Option<String>,
    #[serde(rename = "giftCombo")]
    pub enable_gift_combo: bool,
    #[serde(rename = "comboInterval")]
    pub gift_combo_interval_ms: u64,
    #[serde(rename = "pollInterval")]
    pub poll_interval_ms: u64,
    #[serde(rename = "firefoxCookiesDatabase")]
    pub firefox_cookies_database_path: Option<String>,
}

impl RawConfig {
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open config file");
        let reader = BufReader::new(file);
        let config_file = serde_json::de::from_reader(reader);
        config_file.expect("Failed to deserialize config file")
    }
    pub fn from_args(args: Vec<String>) -> Self {
        // config file
        let path = read_after(&args, vec!["--config"]);
        if let Some(path) = path {
            return RawConfig::from_file(path);
        }
        // room_id
        let room_id: u64 = read_after(&args, vec!["--room-id"])
            .expect("Room ID is required")
            .parse()
            .expect("Invalid room ID");
        // uid
        let uid: Option<u64> = read_after(&args, vec!["--uid"])
            .map(|uid| uid.parse().expect("Invalid user UID"));
        // sessdata
        let sessdata: Option<String> = read_after(&args, vec!["--sessdata"])
            .and_then(|data| {
                if data == "-" {
                    // Read from stdio for better credential security
                    let line = std::io::stdin().lock().lines().next().unwrap().expect("IO Error");
                    Some(line.trim().to_string())
                } else {
                    Some(data.clone())
                }
            });
        // firefox database
        let database_path: Option<String> = read_after(&args, vec!["--firefox-database", "--database"])
            .and_then(|path| Some(path.clone()));
        // gift combo feature
        let enable_gift_combo: bool = args.contains(&"--gift-combo".to_string());
        let gift_combo_interval_ms: u64 = read_after(&args, vec!["--combo-interval"])
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(2000);
        // poll interval
        let poll_interval_ms: u64 = read_after(&args, vec!["--poll-interval"])
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(200);
        // Construct
        RawConfig {
            room_id,
            uid,
            sessdata,
            enable_gift_combo,
            gift_combo_interval_ms,
            poll_interval_ms,
            firefox_cookies_database_path: database_path
        }
    }
}

impl Into<Config> for RawConfig {
    fn into(self) -> Config {
        let mut sessdata = self.sessdata;
        if sessdata.is_none() { if let Some(path) = self.firefox_cookies_database_path {
            //  THIS IS A WORKAROUND
            //  Firefox cookies database is locked by Firefox process.
            //  This code will copy the database file to cwd, then read copied database file.
            const CWD_DATABASE_PATH: &str = "cookies-temp.sqlite";
            std::fs::copy(path, CWD_DATABASE_PATH).expect("Failed to copy database file");

            let conn = Connection::open_with_flags(
                CWD_DATABASE_PATH,
                OpenFlags::SQLITE_OPEN_READ_ONLY
            ).expect("Failed to open database file");

            let result = conn.query_row(
                "SELECT value FROM moz_cookies WHERE host = '.bilibili.com' and name = 'SESSDATA'", [],
                |row| row.get::<usize, String>(0)
            ).expect("Failed to read SESSDATA from database");
            sessdata = Some(result);
            conn.close().expect("Failed to close database file");

            log::debug!("Using SESSDATA from firefox database.")
        }}

        Config {
            room_id:                    self.room_id,
            uid:                        self.uid,
            sessdata:                   sessdata,
            enable_gift_combo:          self.enable_gift_combo,
            gift_combo_interval_ms:     self.gift_combo_interval_ms,
            poll_interval_ms:           self.poll_interval_ms,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub room_id: u64,
    pub uid: Option<u64>,
    pub sessdata: Option<String>,
    pub enable_gift_combo: bool,
    pub gift_combo_interval_ms: u64,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_args(path: Vec<String>) -> Self {
        RawConfig::from_args(path).into()
    }
}

fn read_after<'a>(args: &'a Vec<String>, keys: Vec<&str>) -> Option<&'a String> {
    args.iter().enumerate().find_map(|(index, label)| {
        if keys.iter().any(|key| label.eq(key)) {
            args.get(index + 1)
        } else {
            None
        }
    })
}