use std::io::BufRead;

#[derive(Debug, Clone)]
pub struct Config {
    pub room_id: u64,
    pub uid: Option<u64>,
    pub sessdata: Option<String>,
    pub enable_gift_combo: bool,
    pub gift_combo_interval_ms: u64,
    pub gift_combo_refresh: bool,
    pub poll_interval_ms: u64,
    pub bot_token: String,
    pub chat_id: i64,
}

impl Config {
    pub fn from_args(args: Vec<String>) -> Self {

        fn read_after<'a>(args: &'a Vec<String>, key: &str) -> Option<&'a String> {
            args.iter().enumerate().find_map(|(index, label)| {
                if label.eq(key) {
                    args.get(index + 1)
                } else {
                    None
                }
            })
        }
        // room_id
        let room_id: u64 = read_after(&args, "--room-id")
            .expect("Room ID is required")
            .parse()
            .expect("Invalid room ID");
        // uid
        let uid: Option<u64> = read_after(&args, "--uid").map(|uid| uid.parse().expect("Invalid user UID"));
        // sessdata
        let sessdata: Option<String> = read_after(&args, "--sessdata")
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
        let gift_combo_interval_ms: u64 = read_after(&args, "--combo-interval")
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(2000);
        let gift_combo_refresh: bool = args.contains(&"--refresh-combo".to_string());
        // poll interval
        let poll_interval_ms: u64 = read_after(&args, "--poll-interval")
            .map(|interval| interval.parse().expect("Invalid interval time")).unwrap_or(200);
        // telegram chat id
        let chat_id: i64 = read_after(&args, "--chat_id").map(
            |uid| uid.parse().expect("Invalid Chat ID")
        ).expect("Chat ID is required");
        // bot token
        let bot_token: String = read_after(&args, "--bot_token").expect("Bot Token is required").clone();
        // Construct
        Config {
            room_id,
            uid,
            sessdata,
            enable_gift_combo,
            gift_combo_interval_ms,
            gift_combo_refresh,
            poll_interval_ms,
            bot_token,
            chat_id,
        }
    }
}
