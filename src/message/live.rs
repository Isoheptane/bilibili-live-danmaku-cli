use super::RawLiveMessage;

#[derive(Debug)]
pub struct LiveStartInfo {
    pub room_id: u64
}

#[derive(Debug)]
pub struct LiveStopInfo {
    /*
        For Live Offline Message, roomid can be a string
    */
    pub room_id: String
}

impl TryFrom<RawLiveMessage> for LiveStartInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            LiveStartInfo {
                room_id: value.room_id.ok_or(())?.as_u64().ok_or(())?
            }
        )
    }
}

impl TryFrom<RawLiveMessage> for LiveStopInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            LiveStopInfo {
                room_id: value.room_id.ok_or(())?.as_str().ok_or(())?.to_string()
            }
        )
    }
}