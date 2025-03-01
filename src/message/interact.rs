use crate::message::data::UserInfo;

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

impl InteractInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;

        let uinfo = data.get("uinfo")?;
        let user = UserInfo::try_from(uinfo)?;

        let interact_type = match data.get("msg_type")?.as_u64()? {
            1 => InteractType::Enter,
            2 => InteractType::Follow,
            3 => InteractType::Share,
            4 => InteractType::SpecialFollow,
            5 => InteractType::MutualFollow,
            _ => { return None }
        };
        Some(
            InteractInfo {
                user,
                interact_type
            }
        )
    }
}