use super::RawLiveMessage;

#[derive(Debug)]
pub struct SendGiftInfo {
    pub user_id: u64,
    pub username: String,
    pub gift_name: String,
    pub count: u64
}

impl TryFrom<RawLiveMessage> for SendGiftInfo {
    type Error = ();
    
    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let data = value.data.ok_or(())?;
        let user_id = data.get("uid").ok_or(())?.as_u64().ok_or(())?;
        let username = data.get("uname").ok_or(())?.as_str().ok_or(())?;
        let gift_name = data.get("giftName").ok_or(())?.as_str().ok_or(())?;
        let count = data.get("num").ok_or(())?.as_u64().ok_or(())?;
        Ok(
            SendGiftInfo {
                user_id,
                username: username.to_string(),
                gift_name: gift_name.to_string(),
                count
            }
        )
    }
}

#[derive(Debug)]
pub struct GiftRankInfo {
    pub user_id: u64,
    pub username: String,
    pub coin: u64,
}

#[derive(Debug)]
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