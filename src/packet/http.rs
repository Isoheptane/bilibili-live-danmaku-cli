use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HttpAPIResponse<T> {
    code: isize,
    message: String,
    data: T
}

#[allow(dead_code, unused)]
impl<T> HttpAPIResponse<T> {
    pub fn code(&self) -> isize {
        self.code
    }
    pub fn ok(&self) -> bool {
        self.code == 0
    }
    pub fn response_data(self) -> T {
        self.data
    }
}

#[derive(Debug, Deserialize_repr, Serialize_repr, Clone)]
#[repr(u8)]
pub enum LiveStatus {
    Offline = 0,
    Online = 1,
    Carousel = 2,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoomInitData {
    pub room_id: u64,
    pub short_id: u64,
    pub uid: u64,
    pub live_status: LiveStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebsocketHost {
    pub host: String,
    pub port: u16,
    pub wss_port: u16,
    pub ws_port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DanmakuInfoData {
    pub token: String,
    pub host_list: Vec<WebsocketHost>
}