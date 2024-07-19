use super::RawLiveMessage;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WarningInfo {
    pub message: String
}

impl TryFrom<RawLiveMessage> for WarningInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        Ok(
            WarningInfo {
                message: value.msg.ok_or(())?.to_string()
            }
        )
    }
}