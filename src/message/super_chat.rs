use super::RawLiveMessage;
use super::data::{UserInfo, GuardLevel};

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
        let data = value.data.ok_or(())?;

        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let user_info = data.get("user_info").ok_or(())?;
        let username = user_info.get("uname").ok_or(())?.as_str().ok_or(())?;
        let guard_level: Option<GuardLevel> = user_info.get("guard_level").ok_or(())?.as_u64().ok_or(())?.try_into().ok();
        let user = UserInfo {
            uid: user_id,
            username: username.to_string(),
            guard_level,
            medal: None,
        };

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