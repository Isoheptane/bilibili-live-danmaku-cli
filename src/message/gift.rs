use super::RawLiveMessage;
use super::data::UserInfo;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SendGiftInfo {
    pub user: UserInfo,
    pub gift_name: String,
    pub count: u64
}

impl TryFrom<RawLiveMessage> for SendGiftInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;

        let uinfo = data.get("sender_uinfo").ok_or(())?;
        let user = UserInfo::try_from(uinfo)?;

        let gift_name = data.get("giftName").ok_or(())?.as_str().ok_or(())?;
        let count = data.get("num").ok_or(())?.as_u64().ok_or(())?;
        Ok(
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

impl TryFrom<RawLiveMessage> for GiftTopInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let data = data.as_array().ok_or(())?;
        let mut ranks: Vec<GiftRankInfo> = vec![];
        for value in data {
            let rank = GiftRankInfo {
                user_id: value.get("uid").ok_or(())?.as_u64().ok_or(())?,
                username: value.get("uname").ok_or(())?.as_str().ok_or(())?.to_string(),
                coin: value.get("coin").ok_or(())?.as_u64().ok_or(())?
            };
            ranks.push(rank);
        }
        Ok(
            GiftTopInfo {
                ranks
            }
        )
    }
}