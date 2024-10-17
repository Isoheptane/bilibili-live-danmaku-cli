use super::RawLiveMessage;
use super::data::UserInfo;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SuperChatInfo {
    pub user: UserInfo,
    pub message: String,
    pub price: f64,
    pub keep_time: u64
}

impl TryFrom<RawLiveMessage> for SuperChatInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        log::warn!("{:#?}", value);
        
        let data = value.data.ok_or(())?;

        let uinfo = data.get("uinfo").ok_or(())?;
        let user = UserInfo::try_from(uinfo)?;

        let message = data.get("message").ok_or(())?.as_str().ok_or(())?;
        let price = data.get("price").ok_or(())?.as_f64().ok_or(())?;
        let keep_time = data.get("time").ok_or(())?.as_u64().ok_or(())?;
        Ok(
            SuperChatInfo {
                user,
                message: message.to_string(),
                price,
                keep_time
            }
        )
    }
}