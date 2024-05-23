use serde_json::Value;

use super::RawLiveMessage;

#[derive(Debug)]
pub struct DanmakuInfo {
    pub user_id: u64,
    pub username: String,
    pub is_admin: bool,
    pub is_vip: bool,
    pub guard_level: u64,
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
        let is_admin = user_info.get(2).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let is_vip = user_info.get(3).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let guard_level = info.get(7).ok_or(())?.as_u64().ok_or(())?;
        Ok(
            DanmakuInfo {
                user_id,
                username: username.to_string(),
                text: text.to_string(),
                is_admin,
                is_vip,
                guard_level
            }
        )
    }
}