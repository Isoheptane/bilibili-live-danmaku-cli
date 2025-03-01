use super::RawLiveMessage;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LiveStartInfo {
    pub room_id: u64
}

impl LiveStartInfo {
    
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        Some(
            LiveStartInfo {
                room_id: value.room_id?.as_u64()?
            }
        )
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LiveStopInfo {
    /*
        For Live Offline Message, roomid can be a string
    */
    pub room_id: String
}

impl LiveStopInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        Some(
            LiveStopInfo {
                room_id: value.room_id?.as_str()?.to_string()
            }
        )
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LiveCutOffInfo {
    pub message: String
}

impl LiveCutOffInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        Some(
            LiveCutOffInfo {
                message: value.msg?.to_string()
            }
        )
    }
}