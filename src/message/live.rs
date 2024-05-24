use super::RawLiveMessage;

#[derive(Debug)]
pub struct LiveStartInfo {
    pub room_id: u64
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

#[derive(Debug)]
pub struct LiveStopInfo {
    /*
        For Live Offline Message, roomid can be a string
    */
    pub room_id: String
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

#[derive(Debug)]
pub struct LiveCutOffInfo {
    pub message: String
}

impl TryFrom<RawLiveMessage> for LiveCutOffInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            LiveCutOffInfo {
                message: value.msg.ok_or(())?.to_string()
            }
        )
    }
}