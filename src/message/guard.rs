use super::RawLiveMessage;
use super::data::{UserInfo, GuardLevel};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GuardBuyInfo {
    pub user: UserInfo,
    pub guard_level: GuardLevel,
    pub count: u64,
}

impl TryFrom<RawLiveMessage> for GuardBuyInfo {
    type Error = ();

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;

        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("username").ok_or(())?.as_str().ok_or(())?;
        let guard_level: GuardLevel = data.get("guard_level").ok_or(())?.as_u64().ok_or(())?.try_into()?;
        let count = data.get("num").ok_or(())?.as_u64().ok_or(())?;
        let user = UserInfo {
            uid: user_id,
            username: username.to_string(),
            guard_level: Some(guard_level),
            medal: None
        };

        Ok(
            GuardBuyInfo {
                user,
                guard_level,
                count
            }
        )
    }
}