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
    pub user_id: u64,
    pub username: String,
    pub interact_type: InteractType
}

impl TryFrom<RawLiveMessage> for InteractInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("uname").ok_or(())?.as_str().ok_or(())?;
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
                user_id,
                username: username.to_string(),
                interact_type
            }
        )
    }
}