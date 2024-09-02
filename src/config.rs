use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "roomId")]
    pub room_id: u64,
    #[serde(rename = "uid")]
    pub uid: Option<u64>,
    pub sessdata: Option<String>,
    #[serde(rename = "giftCombo")]
    pub enable_gift_combo: bool,
    #[serde(rename = "comboInterval")]
    pub gift_combo_interval_ms: u64,
    #[serde(rename = "comboRefresh")]
    pub gift_combo_refresh: bool,
    #[serde(rename = "pollInterval")]
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open config file");
        let reader = BufReader::new(file);
        let config = serde_json::de::from_reader(reader);
        config.expect("Failed to deserialize config file")
    }
    pub fn from_args(args: Vec<String>) -> Self {
        // config file
        let path = read_after(&args, vec!["--config"]);
        if let Some(path) = path {
            return Config::from_file(path);
        }
        // room_id
        let room_id: u64 = read_after(&args, vec!["--room-id"])
            .expect("Room ID is required")
            .parse()
            .expect("Invalid room ID");
        // uid
        let uid: Option<u64> = read_after(&args, vec!["--uid"]).map(|uid| uid.parse().expect("Invalid user UID"));
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
        // gift combo feature
        let enable_gift_combo: bool = args.contains(&"--gift-combo".to_string());
        let gift_combo_interval_ms: u64 = read_after(&args, vec!["--combo-interval"])
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(2000);
        let gift_combo_refresh: bool = args.contains(&"--refresh-combo".to_string());
        // poll interval
        let poll_interval_ms: u64 = read_after(&args, vec!["--poll-interval"])
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(200);
        // Construct
        Config {
            room_id,
            uid,
            sessdata,
            enable_gift_combo,
            gift_combo_interval_ms,
            gift_combo_refresh,
            poll_interval_ms
        }
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