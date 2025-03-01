use super::RawLiveMessage;
use super::data::{UserInfo, GuardLevel};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GuardBuyInfo {
    pub user: UserInfo,
    pub guard_level: GuardLevel,
    pub count: u64,
}

impl GuardBuyInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;

        let user_id = data.get("uid")?.as_u64()?;
        let username = data.get("username")?.as_str()?;
        let guard_level: GuardLevel = data.get("guard_level")?.as_u64()?.try_into().ok()?;
        let count = data.get("num")?.as_u64()?;
        let user = UserInfo {
            uid: user_id,
            username: username.to_string(),
            guard_level: Some(guard_level),
            medal: None
        };

        Some(
            GuardBuyInfo {
                user,
                guard_level,
                count
            }
        )
    }
}