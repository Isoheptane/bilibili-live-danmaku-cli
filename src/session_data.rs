use std::error;
use derive_more::Display;

use colored::Colorize;
use crate::{DanmakuInfoData, HttpAPIResponse, RoomInitData, WebsocketHost};

#[derive(Clone)]
pub struct SessionData {
    pub room_id: u64,
    pub uid: u64,
    pub token: String
}

#[derive(Debug, Display)]
pub enum InitRoomError {
    RequestFailed(ureq::Error),
    BadResponse(std::io::Error),
}

impl error::Error for InitRoomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::RequestFailed(e) => Some(e),
            Self::BadResponse(e) => Some(e),
        }
    }
}

pub fn init_room_data(
    room_id: u64,
    uid: Option<u64>,
    sessdata: Option<String>
) -> Result<(SessionData, Vec<WebsocketHost>), InitRoomError> {
    // Start calling APIs
    let agent = ureq::builder().tls_connector(native_tls::TlsConnector::new().unwrap().into()).build();
    // Get room data for the real room id
    let room_data: RoomInitData = agent.get(
        &format!("https://api.live.bilibili.com/room/v1/Room/room_init?id={}", room_id)
    )
        .call().map_err(|e| InitRoomError::RequestFailed(e))?
        .into_json::<HttpAPIResponse<RoomInitData>>().map_err(|e| InitRoomError::BadResponse(e))?
        .response_data();

    let room_id = room_data.room_id;
    log::debug!(
        target: "main",
        "Requested real room ID: {}", room_id.to_string().bright_green()
    );
    // Get danmaku info data
    let danmaku_info_data: DanmakuInfoData = agent.get(
            &format!("https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}", room_id)
    )
        .set("Cookie", format!("SESSDATA={}", sessdata.unwrap_or_default()).as_str())
        .call().map_err(|e| InitRoomError::RequestFailed(e))?
        .into_json::<HttpAPIResponse<DanmakuInfoData>>().map_err(|e| InitRoomError::BadResponse(e))?
        .response_data();

    log::debug!(
        target: "main",
        "Requested token and WebSocket servers. {} servers available",
        danmaku_info_data.host_list.len().to_string().bright_green()
    );

    let token = danmaku_info_data.token;
    return Ok((
        SessionData { room_id, uid: uid.unwrap_or(0), token },
        danmaku_info_data.host_list
    ));
}