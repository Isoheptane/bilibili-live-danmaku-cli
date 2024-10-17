use serde_json::Value;

use super::{guard::GuardLevel, MedalInfo};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub uid: u64,
    pub username: String,
    pub guard_level: Option<GuardLevel>,
    pub medal: Option<MedalInfo>,
}

impl TryFrom<&Value> for UserInfo {
    type Error = ();
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let uid = value.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let user_name = value.get("base").ok_or(())?.get("name").ok_or(())?.as_str().ok_or(())?;
        let guard_info = value.get("guard").ok_or(())?.as_object();
        let guard_level: Option<GuardLevel> = match guard_info {
            None => None,
            Some(guard_info) => guard_info.get("level").ok_or(())?.as_u64().ok_or(())?.try_into().ok()
        };
        let medal_info = value.get("medal").ok_or(())?.as_object();
        let medal = match medal_info {
            None => None,
            Some(medal_info) => {
                let medal_name = medal_info.get("name").ok_or(())?.as_str().ok_or(())?;
                let medal_level = medal_info.get("level").ok_or(())?.as_u64().ok_or(())?;
                let medal_uid = medal_info.get("ruid").ok_or(())?.as_u64().ok_or(())?;
                Some (MedalInfo {
                    medal_name: medal_name.to_string(),
                    level: medal_level,
                    user_id: medal_uid,
                    username: "Unknown".to_string()
                })
            }
        };
        Ok(
            UserInfo {
                uid,
                username: user_name.to_string(),
                guard_level,
                medal
            }
        )
    }
}