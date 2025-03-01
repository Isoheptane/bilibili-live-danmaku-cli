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

impl SuperChatInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        log::warn!("{:#?}", value);
        
        let data = value.data?;

        let uinfo = data.get("uinfo")?;
        let user = UserInfo::try_from(uinfo)?;

        let message = data.get("message")?.as_str()?;
        let price = data.get("price")?.as_f64()?;
        let keep_time = data.get("time")?.as_u64()?;
        Some(
            SuperChatInfo {
                user,
                message: message.to_string(),
                price,
                keep_time
            }
        )
    }
}