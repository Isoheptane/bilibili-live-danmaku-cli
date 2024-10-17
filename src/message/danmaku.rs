use serde_json::Value;

use super::RawLiveMessage;
use super::data::UserInfo;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct DanmakuInfo {
    pub user: UserInfo,
    pub is_admin: bool,
    pub is_vip: bool,
    pub text: String,
}

impl TryFrom<RawLiveMessage> for DanmakuInfo {
    type Error = ();

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let info = value.info.ok_or(())?;
        
        let uinfo = info.get(0).ok_or(())?.get(15).ok_or(())?.get("user").ok_or(())?;
        let user = UserInfo::try_from(uinfo)?;

        let text = info.get(1).ok_or(())?.as_str().ok_or(())?;

        let user_info: &Vec<Value> = info.get(2).ok_or(())?.as_array().ok_or(())?;
        let is_admin = user_info.get(2).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let is_vip = user_info.get(3).ok_or(())?.as_u64().is_some_and(|value| value == 1);

        let danmaku_info = DanmakuInfo {
            user,
            text: text.to_string(),
            is_admin,
            is_vip,
        };

        log::debug!("Danmaku Received: {:#?}", danmaku_info);
        Ok(danmaku_info)
    }
}