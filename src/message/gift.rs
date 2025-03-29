use super::RawLiveMessage;
use super::data::{GuardLevel, UserInfo};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SendGiftInfo {
    pub user: UserInfo,
    pub gift_name: String,
    pub count: u64
}

impl SendGiftInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;

        let uinfo = data.get("sender_uinfo")?;
        let user = UserInfo::try_from(uinfo)?;

        let guard_level: Option<GuardLevel> = data.get("guard_level")?.as_u64()?.try_into().ok();
        let user = user.set_guard(guard_level);

        let gift_name = data.get("giftName")?.as_str()?;
        let count = data.get("num")?.as_u64()?;
        Some(
            SendGiftInfo {
                user,
                gift_name: gift_name.to_string(),
                count
            }
        )
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GiftRankInfo {
    pub user_id: u64,
    pub username: String,
    pub coin: u64,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GiftTopInfo {
    pub ranks: Vec<GiftRankInfo>
}

impl GiftTopInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        let data = value.data?;
        let data = data.as_array()?;
        let mut ranks: Vec<GiftRankInfo> = vec![];
        for value in data {
            let rank = GiftRankInfo {
                user_id: value.get("uid")?.as_u64()?,
                username: value.get("uname")?.as_str()?.to_string(),
                coin: value.get("coin")?.as_u64()?
            };
            ranks.push(rank);
        }
        Some(
            GiftTopInfo {
                ranks
            }
        )
    }
}