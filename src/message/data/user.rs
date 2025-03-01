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

impl UserInfo {
    
    pub fn try_from(value: &Value) -> Option<Self> {
        let uid = value.get("uid")?.as_u64()?;
        let user_name = value.get("base")?.get("name")?.as_str()?;
        let guard_info = value.get("guard")?.as_object();
        let guard_level: Option<GuardLevel> = match guard_info {
            None => None,
            Some(guard_info) => guard_info.get("level")?.as_u64()?.try_into().ok()
        };
        let medal_info = value.get("medal")?.as_object();
        let medal = match medal_info {
            None => None,
            Some(medal_info) => {
                let medal_name = medal_info.get("name")?.as_str()?;
                let medal_level = medal_info.get("level")?.as_u64()?;
                let medal_uid = medal_info.get("ruid")?.as_u64()?;
                Some (MedalInfo {
                    medal_name: medal_name.to_string(),
                    level: medal_level,
                    user_id: medal_uid,
                    username: "Unknown".to_string()
                })
            }
        };
        Some(
            UserInfo {
                uid,
                username: user_name.to_string(),
                guard_level,
                medal
            }
        )
    }
}