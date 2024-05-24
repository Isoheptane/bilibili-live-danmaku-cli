use super::RawLiveMessage;

#[derive(Debug, Clone, Copy)]
pub enum GuardLevel {
    Captain     = 1,    // 艦長
    Commander   = 2,    // 提督
    Governor    = 3,    // 總督
}

impl TryFrom<u64> for GuardLevel {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(GuardLevel::Captain),
            2 => Ok(GuardLevel::Commander),
            3 => Ok(GuardLevel::Governor),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct GuardBuyInfo {
    pub user_id: u64,
    pub username: String,
    pub guard_level: GuardLevel,
    pub count: u64,
}

impl TryFrom<RawLiveMessage> for GuardBuyInfo {
    type Error = ();

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("user_info").ok_or(())?.get("username").ok_or(())?.as_str().ok_or(())?;
        let guard_level: GuardLevel = data.get("guard_level").ok_or(())?.as_u64().ok_or(())?.try_into()?;
        let count = data.get("guard_level").ok_or(())?.as_u64().ok_or(())?;
        Ok(
            GuardBuyInfo {
                user_id,
                username: username.to_string(),
                guard_level,
                count
            }
        )
    }
}