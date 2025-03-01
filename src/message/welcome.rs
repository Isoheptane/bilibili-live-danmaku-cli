use super::RawLiveMessage;
use super::data::GuardLevel;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WelcomeInfo {
    pub user_id: u64,
    pub username: String,
    pub is_admin: bool,
}

impl WelcomeInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;
        let user_id = data.get("uid")?.as_u64()?;
        let username = data.get("uname")?.as_str()?;
        let is_admin = data.get("isadmin")?.as_u64().is_some_and(|value| value == 1);
        Some(
            WelcomeInfo {
                user_id,
                username: username.to_string(),
                is_admin
            }
        )
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WelcomeGuardInfo {
    pub user_id: u64,
    pub username: String,
    pub guard_level: Option<GuardLevel>,
}

impl WelcomeGuardInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;
        let user_id = data.get("uid")?.as_u64()?;
        let username = data.get("uname")?.as_str()?;
        let guard_level: Option<GuardLevel> = data.get("guard_level")?.as_u64()?.try_into().ok();
        Some(
            WelcomeGuardInfo {
                user_id,
                username: username.to_string(),
                guard_level
            }
        )
    }
}