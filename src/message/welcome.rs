use super::{guard::GuardLevel, RawLiveMessage};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WelcomeInfo {
    pub user_id: u64,
    pub username: String,
    pub is_admin: bool,
}

impl TryFrom<RawLiveMessage> for WelcomeInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("uname").ok_or(())?.as_str().ok_or(())?;
        let is_admin = data.get("isadmin").ok_or(())?.as_u64().is_some_and(|value| value == 1);
        Ok(
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

impl TryFrom<RawLiveMessage> for WelcomeGuardInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("uname").ok_or(())?.as_str().ok_or(())?;
        let guard_level: Option<GuardLevel> = data.get("guard_level").ok_or(())?.as_u64().ok_or(())?.try_into().ok();
        Ok(
            WelcomeGuardInfo {
                user_id,
                username: username.to_string(),
                guard_level
            }
        )
    }
}