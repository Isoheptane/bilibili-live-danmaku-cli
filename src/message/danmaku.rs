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

impl DanmakuInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let info = value.info?;
        
        let uinfo = info.get(0)?.get(15)?.get("user")?;
        let user = UserInfo::try_from(uinfo)?;

        let text = info.get(1)?.as_str()?;

        let user_info: &Vec<Value> = info.get(2)?.as_array()?;
        let is_admin = user_info.get(2)?.as_u64().is_some_and(|value| value == 1);
        let is_vip = user_info.get(3)?.as_u64().is_some_and(|value| value == 1);

        let danmaku_info = DanmakuInfo {
            user,
            text: text.to_string(),
            is_admin,
            is_vip,
        };

        log::debug!("Danmaku Received: {:#?}", danmaku_info);
        Some(danmaku_info)
    }
}