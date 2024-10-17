use serde_json::Value;

use super::RawLiveMessage;
use super::data::{MedalInfo, UserInfo, GuardLevel};

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

        let text = info.get(1).ok_or(())?.as_str().ok_or(())?;
        
        let user_info: &Vec<Value> = info.get(2).ok_or(())?.as_array().ok_or(())?;
        let user_id = user_info.get(0).ok_or(())?.as_u64().ok_or(())?;
        let username = user_info.get(1).ok_or(())?.as_str().ok_or(())?;
        let guard_level: Option<GuardLevel> = info.get(7).ok_or(())?.as_u64().ok_or(())?.try_into().ok();
        let is_admin = user_info.get(2).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let is_vip = user_info.get(3).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let medal_info: &Vec<Value> = info.get(3).ok_or(())?.as_array().ok_or(())?;
        let medal = if medal_info.is_empty() {
            None
        } else {
            Some(
                MedalInfo {
                    medal_name: medal_info.get(1).ok_or(())?.as_str().ok_or(())?.to_string(),
                    level: medal_info.get(0).ok_or(())?.as_u64().ok_or(())?,
                    username: medal_info.get(2).ok_or(())?.as_str().ok_or(())?.to_string(),
                    user_id: medal_info.get(12).ok_or(())?.as_u64().ok_or(())?,
                }
            )
        };

        let user = UserInfo {
            uid: user_id,
            username: username.to_string(),
            guard_level,
            medal
        };

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