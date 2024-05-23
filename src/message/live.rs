use super::RawLiveMessage;

#[derive(Debug)]
pub struct LiveOnlineInfo {
    pub room_id: u64
}

#[derive(Debug)]
pub struct LiveOfflineInfo {
    pub room_id: u64
}

impl TryFrom<RawLiveMessage> for LiveOnlineInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            LiveOnlineInfo {
                room_id: value.room_id.ok_or(())?
            }
        )
    }
}

impl TryFrom<RawLiveMessage> for LiveOfflineInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            LiveOfflineInfo {
                room_id: value.room_id.ok_or(())?
            }
        )
    }
}