use serde_json::Value;

use super::{guard::GuardLevel, RawLiveMessage};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct DanmakuInfo {
    pub user_id: u64,
    pub username: String,
    pub is_admin: bool,
    pub is_vip: bool,
    pub guard_level: Option<GuardLevel>,
    pub text: String,
    pub badge: Option<BadgeInfo>
}

impl TryFrom<RawLiveMessage> for DanmakuInfo {
    type Error = ();

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        // log::info!("{}", value);
        let info = value.info.ok_or(())?;

        let text = info.get(1).ok_or(())?.as_str().ok_or(())?;
        
        let user_info: &Vec<Value> = info.get(2).ok_or(())?.as_array().ok_or(())?;
        let user_id = user_info.get(0).ok_or(())?.as_u64().ok_or(())?;
        let username = user_info.get(1).ok_or(())?.as_str().ok_or(())?;
        let is_admin = user_info.get(2).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let is_vip = user_info.get(3).ok_or(())?.as_u64().is_some_and(|value| value == 1);
        let guard_level: Option<GuardLevel> = info.get(7).ok_or(())?.as_u64().ok_or(())?.try_into().ok();

        let badge_info: &Vec<Value> = info.get(3).ok_or(())?.as_array().ok_or(())?;
        let badge = if badge_info.is_empty() {
            None
        } else {
            Some(
                BadgeInfo {
                    badge_name: badge_info.get(1).ok_or(())?.as_str().ok_or(())?.to_string(),
                    level: badge_info.get(0).ok_or(())?.as_u64().ok_or(())?,
                    username: badge_info.get(2).ok_or(())?.as_str().ok_or(())?.to_string(),
                    user_id: badge_info.get(12).ok_or(())?.as_u64().ok_or(())?,
                }
            )
        };

        let danmaku_info = DanmakuInfo {
            user_id,
            username: username.to_string(),
            text: text.to_string(),
            is_admin,
            is_vip,
            guard_level,
            badge
        };
        log::debug!("Danmaku Received: {:#?}", danmaku_info);
        Ok(danmaku_info)
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct BadgeInfo {
    pub badge_name: String,
    pub level: u64,
    pub user_id: u64,
    pub username: String,
}