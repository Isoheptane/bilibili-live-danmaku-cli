use derive_more::Display;
use md5;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::collections::BTreeMap;
use std::error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{DanmakuInfoData, HttpAPIResponse, RoomInitData, WebsocketHost};
use colored::Colorize;

const WBI_CACHE_DIR: &str = ".wbi_cache";
const WBI_CACHE_DURATION: u64 = 12 * 60 * 60; // 12 hours in seconds

const MIXIN_KEY_ENC_TAB: [u8; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

fn gen_mixin_key(raw_wbi_key: impl AsRef<[u8]>) -> String {
    let raw_wbi_key = raw_wbi_key.as_ref();
    let mut mixin_key = {
        let binding = MIXIN_KEY_ENC_TAB
            .iter()
            .map(|n| raw_wbi_key[*n as usize])
            .collect::<Vec<u8>>();
        unsafe { String::from_utf8_unchecked(binding) }
    };
    let _ = mixin_key.split_off(32); // 截取前 32 位字符
    mixin_key
}

#[derive(Clone)]
pub struct SessionData {
    pub room_id: u64,
    pub uid: u64,
    pub token: String,
}

#[derive(Debug, Display)]
pub enum InitRoomError {
    RequestFailed(ureq::Error),
    BadResponse(std::io::Error),
    IoError(io::Error),
}

impl error::Error for InitRoomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::RequestFailed(e) => Some(e),
            Self::BadResponse(e) => Some(e),
            Self::IoError(e) => Some(e),
        }
    }
}

impl From<io::Error> for InitRoomError {
    fn from(err: io::Error) -> Self {
        InitRoomError::IoError(err)
    }
}

fn get_wbi_keys(agent: &ureq::Agent) -> Result<(String, String), InitRoomError> {
    // Create cache directory if it doesn't exist
    fs::create_dir_all(WBI_CACHE_DIR)?;

    let img_key_path = Path::new(WBI_CACHE_DIR).join("img_key");
    let sub_key_path = Path::new(WBI_CACHE_DIR).join("sub_key");
    let timestamp_path = Path::new(WBI_CACHE_DIR).join("timestamp");

    // Check if we have cached keys and if they're still valid
    if img_key_path.exists() && sub_key_path.exists() && timestamp_path.exists() {
        let timestamp = fs::read_to_string(&timestamp_path)?
            .parse::<u64>()
            .unwrap_or(0);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time - timestamp < WBI_CACHE_DURATION {
            // Cache is still valid, read the keys
            let img_key = fs::read_to_string(&img_key_path)?;
            let sub_key = fs::read_to_string(&sub_key_path)?;
            log::debug!(
                target: "main",
                "Using cached WBI keys - img_key: {}, sub_key: {}",
                img_key.bright_green(),
                sub_key.bright_green()
            );
            return Ok((img_key, sub_key));
        }
    }

    // Cache is invalid or doesn't exist, get new keys
    log::debug!(
        target: "main",
        "Requesting WBI keys from nav API..."
    );

    let nav_data: serde_json::Value = agent
        .get("https://api.bilibili.com/x/web-interface/nav")
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .set("Referer", "https://www.bilibili.com/")
        .call()
        .map_err(|e| InitRoomError::RequestFailed(e))?
        .into_json::<HttpAPIResponse<serde_json::Value>>()
        .map_err(|e| InitRoomError::BadResponse(e))?
        .response_data();

    let wbi_img = nav_data.get("wbi_img").ok_or_else(|| {
        log::error!(
            target: "main",
            "Missing wbi_img in nav response. Full response: {:#?}",
            nav_data
        );
        InitRoomError::BadResponse(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Missing wbi_img in nav response",
        ))
    })?;

    let img_url = wbi_img
        .get("img_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            log::error!(
                target: "main",
                "Missing img_url in wbi_img. WBI img data: {:#?}",
                wbi_img
            );
            InitRoomError::BadResponse(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing img_url in wbi_img",
            ))
        })?;

    let sub_url = wbi_img
        .get("sub_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            log::error!(
                target: "main",
                "Missing sub_url in wbi_img. WBI img data: {:#?}",
                wbi_img
            );
            InitRoomError::BadResponse(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing sub_url in wbi_img",
            ))
        })?;

    let img_key = img_url
        .split('/')
        .last()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("");
    let sub_key = sub_url
        .split('/')
        .last()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("");

    // Save the new keys and timestamp
    let mut img_key_file = File::create(&img_key_path)?;
    let mut sub_key_file = File::create(&sub_key_path)?;
    let mut timestamp_file = File::create(&timestamp_path)?;

    img_key_file.write_all(img_key.as_bytes())?;
    sub_key_file.write_all(sub_key.as_bytes())?;
    timestamp_file.write_all(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
            .as_bytes(),
    )?;

    log::debug!(
        target: "main",
        "Cached new WBI keys - img_key: {}, sub_key: {}",
        img_key.bright_green(),
        sub_key.bright_green()
    );

    Ok((img_key.to_string(), sub_key.to_string()))
}

fn url_encode(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC)
        .to_string()
        .replace('+', "%20")
}

fn calculate_w_rid(params: &BTreeMap<&str, String>, mixin_key: &str) -> String {
    // Sort parameters by key and encode values
    let encoded_params: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect();

    // Join parameters with &
    let param_string = encoded_params.join("&");

    // Append mixin_key
    let string_to_hash = format!("{}{}", param_string, mixin_key);

    // Calculate MD5
    let result = md5::compute(string_to_hash.as_bytes());

    // Convert to hex string
    format!("{:x}", result)
}

pub fn init_room_data(
    room_id: u64,
    uid: Option<u64>,
    sessdata: Option<String>,
) -> Result<(SessionData, Vec<WebsocketHost>), InitRoomError> {
    // Start calling APIs
    let agent = ureq::builder()
        .tls_connector(native_tls::TlsConnector::new().unwrap().into())
        .build();
    // Get room data for the real room id
    let room_data: RoomInitData = agent
        .get(&format!(
            "https://api.live.bilibili.com/room/v1/Room/room_init?id={}",
            room_id
        ))
        .call()
        .map_err(|e| InitRoomError::RequestFailed(e))?
        .into_json::<HttpAPIResponse<RoomInitData>>()
        .map_err(|e| InitRoomError::BadResponse(e))?
        .response_data();

    let room_id = room_data.room_id;
    log::debug!(
        target: "main",
        "Requested real room ID: {}", room_id.to_string().bright_green()
    );

    // Get WBI keys (from cache or API)
    let (img_key, sub_key) = get_wbi_keys(&agent)?;

    // Get raw_wbi_key
    let raw_wbi_key = format!("{}{}", img_key, sub_key);

    // Get mixin_key
    let mixin_key = gen_mixin_key(raw_wbi_key.as_bytes());

    // Get wts
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    // Create sorted parameters map
    let mut params = BTreeMap::new();
    params.insert("id", room_id.to_string());
    params.insert("wts", wts);

    // Calculate w_rid
    let w_rid = calculate_w_rid(&params, &mixin_key);

    // Build final query string
    let query_string = format!("id={}&wts={}&w_rid={}", room_id, params["wts"], w_rid);

    // Get danmaku info data
    let danmaku_info_data: DanmakuInfoData = agent
        .get(&format!(
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?{}",
            query_string
        ))
        .set(
            "Cookie",
            format!("SESSDATA={}", sessdata.unwrap_or_default()).as_str(),
        )
        .call()
        .map_err(|e| InitRoomError::RequestFailed(e))?
        .into_json::<HttpAPIResponse<DanmakuInfoData>>()
        .map_err(|e| InitRoomError::BadResponse(e))?
        .response_data();

    let token = danmaku_info_data.token;
    return Ok((
        SessionData {
            room_id,
            uid: uid.unwrap_or(0),
            token,
        },
        danmaku_info_data.host_list,
    ));
}
