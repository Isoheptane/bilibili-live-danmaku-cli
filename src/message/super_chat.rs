use super::RawLiveMessage;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SuperChatInfo {
    pub user_id: u64,
    pub username: String,
    pub message: String,
    pub price: f64,
    pub keep_time: u64
}

impl TryFrom<RawLiveMessage> for SuperChatInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("user_info").ok_or(())?.get("username").ok_or(())?.as_str().ok_or(())?;
        let message = data.get("message").ok_or(())?.as_str().ok_or(())?;
        let price = data.get("price").ok_or(())?.as_f64().ok_or(())?;
        let keep_time = data.get("time").ok_or(())?.as_u64().ok_or(())?;
        Ok(
            SuperChatInfo {
                user_id,
                username: username.to_string(),
                message: message.to_string(),
                price,
                keep_time
            }
        )
    }
}