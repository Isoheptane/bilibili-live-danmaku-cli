mod danmaku;
mod gift;
mod live;
mod warning;
mod welcome;

use derive_more::Display;
use serde::Deserialize;
use serde_json::{Map, Value};

use danmaku::DanmakuInfo;
use gift::SendGiftInfo;
use live::{LiveStartInfo, LiveStopInfo};
use welcome::{WelcomeInfo, WelcomeGuardInfo};

use self::{live::LiveCutOffInfo, warning::WarningInfo};

#[derive(Debug, Deserialize, Clone)]
pub struct RawLiveMessage {
    pub cmd: String,
    /*
        roomid can be either String or Number.
        I definitely don't know why & how does
        the devs in bilibili thinks.
    */
    #[serde(rename = "roomid")]
    pub room_id: Option<Value>,
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
    Danmaku         (DanmakuInfo),
    SendGift        (SendGiftInfo),
    LiveStart       (LiveStartInfo),
    LiveStop        (LiveStopInfo),
    LiveCutOff      (LiveCutOffInfo),
    Welcome         (WelcomeInfo),
    WelcomeGuard    (WelcomeGuardInfo),
    Warning         (WarningInfo)
}

impl TryFrom<RawLiveMessage> for LiveMessage {
    type Error = RawMessageDeserializeError;

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        match value.cmd.as_str() {
            "DANMU_MSG"     => DanmakuInfo::try_from(value).map(|value| Self::Danmaku(value)),
            "SEND_GIFT"     => SendGiftInfo::try_from(value).map(|value| Self::SendGift(value)),
            "LIVE"          => LiveStartInfo::try_from(value).map(|value| Self::LiveStart(value)),
            "PREPARING"     => LiveStopInfo::try_from(value).map(|value| Self::LiveStop(value)),
            "CUT_OFF"       => LiveCutOffInfo::try_from(value).map(|value| Self::LiveCutOff(value)),
            "WELCOME"       => WelcomeInfo::try_from(value).map(|value| Self::Welcome(value)),
            "WELCOME_GUARD" => WelcomeGuardInfo::try_from(value).map(|value| Self::WelcomeGuard(value)),
            "WARNING"       => WarningInfo::try_from(value).map(|value| Self::Warning(value)),
            _ => { return Err(RawMessageDeserializeError::NotSupported(value.cmd)) }
        }
        .map_err(|_| RawMessageDeserializeError::DeserializeError)
    }
}