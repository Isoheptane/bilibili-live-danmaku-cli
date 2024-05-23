mod danmaku;
mod gift;
mod live;
mod welcome;

use derive_more::Display;
use serde::Deserialize;
use serde_json::{Map, Value};

use self::{danmaku::DanmakuInfo, gift::SendGiftInfo, live::{LiveOfflineInfo, LiveOnlineInfo}, welcome::{WelcomeGuardInfo, WelcomeInfo}};

#[derive(Debug, Deserialize, Clone)]
pub struct RawLiveMessage {
    pub cmd: String,
    #[serde(rename = "roomid")]
    pub room_id: Option<u64>,
    pub msg: Option<String>,
    pub info: Option<Vec<Value>>,
    pub data: Option<Map<String, Value>>
}

#[derive(Debug, Display)]
pub enum RawMessageDeserializeError {
    NotSupported(String),
    DeserializeError,
}

impl std::error::Error for RawMessageDeserializeError {
    fn description(&self) -> &str {
        match self {
            Self::NotSupported(_) => "Message type not supported",
            Self::DeserializeError => "Failed to deserialze message"
        }
    }
}


#[derive(Debug)]
pub enum LiveMessage {
    Danmaku (DanmakuInfo),
    SendGift (SendGiftInfo),
    Online (LiveOnlineInfo),
    Offline (LiveOfflineInfo),
    Welcome (WelcomeInfo),
    WelcomeGuard (WelcomeGuardInfo)
}

impl TryFrom<RawLiveMessage> for LiveMessage {
    type Error = RawMessageDeserializeError;

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        match value.cmd.as_str() {
            "DANMU_MSG"     => DanmakuInfo::try_from(value).map(|value| Self::Danmaku(value)),
            "SEND_GIFT"     => SendGiftInfo::try_from(value).map(|value| Self::SendGift(value)),
            "LIVE"          => LiveOnlineInfo::try_from(value).map(|value| Self::Online(value)),
            "PREPARING"     => LiveOfflineInfo::try_from(value).map(|value| Self::Offline(value)),
            "WELCOME"       => WelcomeInfo::try_from(value).map(|value| Self::Welcome(value)),
            "WELCOME_GUARD" => WelcomeGuardInfo::try_from(value).map(|value| Self::WelcomeGuard(value)),
            _ => { return Err(RawMessageDeserializeError::NotSupported(value.cmd)) }
        }
        .map_err(|_| RawMessageDeserializeError::DeserializeError)
    }
}