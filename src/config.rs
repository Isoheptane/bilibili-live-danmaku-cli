pub struct Config {
    pub room_id: u64,
    pub uid: Option<u64>,
    pub sessdata: Option<String>,
}

fn read_after<'a>(args: &'a Vec<String>, key: &str) -> Option<&'a String> {
    args.iter().enumerate().find_map(|(index, label)| {
        if label.eq(key) {
            args.get(index + 1)
        } else {
            None
        }
    })
}

impl Config {
    pub fn from_args(args: Vec<String>) -> Self {
        let room_id: u64 = read_after(&args, "--room-id").unwrap().parse().unwrap();
        let uid: Option<u64> = read_after(&args, "--uid").map(|uid| uid.parse().unwrap());
        let sessdata: Option<String> = read_after(&args, "--sessdata").cloned();
        Config {
            room_id,
            uid,
            sessdata
        }
    }
}
