use crate::message::data::{GuardLevel, UserInfo};

use super::RawLiveMessage;

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum InteractType {
    Enter           = 1,
    Follow          = 2,
    Share           = 3,
    SpecialFollow   = 4,
    MutualFollow    = 5,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct InteractInfo {
    pub user: UserInfo,
    pub interact_type: InteractType
}

impl TryFrom<RawLiveMessage> for InteractInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;

        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("uname").ok_or(())?.as_str().ok_or(())?;
        let guard_level: Option<GuardLevel> = data
            .get("uinfo").ok_or(())?
            .get("guard").ok_or(())?
            .get("level").ok_or(())?
            .as_u64().ok_or(())?.try_into().ok();
        let user = UserInfo {
            uid: user_id,
            username: username.to_string(),
            guard_level,
            medal: None
        };

        let interact_type = match data.get("msg_type").ok_or(())?.as_u64().ok_or(())? {
            1 => InteractType::Enter,
            2 => InteractType::Follow,
            3 => InteractType::Share,
            4 => InteractType::SpecialFollow,
            5 => InteractType::MutualFollow,
            _ => { return Err(()) }
        };
        Ok(
            InteractInfo {
                user,
                interact_type
            }
        )
    }
}