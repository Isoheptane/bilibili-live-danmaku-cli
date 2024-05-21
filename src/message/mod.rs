mod danmaku;
mod gift;

use serde::Deserialize;
use serde_json::{Map, Value};

use self::{danmaku::DanmakuInfo, gift::SendGiftInfo};

#[derive(Debug, Deserialize, Clone)]
pub struct RawLiveMessage {
    pub cmd: String,
    #[serde(rename = "roomid")]
    pub room_id: Option<u64>,
    pub msg: Option<String>,
    pub info: Option<Vec<Value>>,
    pub data: Option<Map<String, Value>>
}

#[derive(Debug)]
pub enum LiveMessage {
    Danmaku (DanmakuInfo),
    SendGift (SendGiftInfo),
}

impl TryFrom<RawLiveMessage> for LiveMessage {
    type Error = ();

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        match value.cmd.as_str() {
            "DANMU_MSG" => DanmakuInfo::try_from(value).map(|value| Self::Danmaku(value)),
            "SEND_GIFT" => SendGiftInfo::try_from(value).map(|value| Self::SendGift(value)),
            _ => Err(())
        }
    }
}