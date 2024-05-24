mod danmaku;
mod gift;
mod interact;
mod live;
mod super_chat;
mod warning;
mod welcome;

use derive_more::Display;
use serde::Deserialize;
use serde_json::{Map, Value};

use danmaku::DanmakuInfo;
use gift::SendGiftInfo;
use live::{LiveStartInfo, LiveStopInfo};
use welcome::{WelcomeInfo, WelcomeGuardInfo};

use self::{interact::InteractInfo, live::LiveCutOffInfo, super_chat::SuperChatInfo, warning::WarningInfo};

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
    LiveStart       (LiveStartInfo),
    LiveStop        (LiveStopInfo),
    LiveCutOff      (LiveCutOffInfo),
    Welcome         (WelcomeInfo),
    WelcomeGuard    (WelcomeGuardInfo),
    Warning         (WarningInfo),
    Danmaku         (DanmakuInfo),
    SendGift        (SendGiftInfo),
    SuperChat       (SuperChatInfo),
    Interact        (InteractInfo),
}

impl TryFrom<RawLiveMessage> for LiveMessage {
    type Error = RawMessageDeserializeError;

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        match value.cmd.as_str() {
            "LIVE"          => LiveStartInfo::try_from(value).map(|value| Self::LiveStart(value)),
            "PREPARING"     => LiveStopInfo::try_from(value).map(|value| Self::LiveStop(value)),
            "WARNING"       => WarningInfo::try_from(value).map(|value| Self::Warning(value)),
            "CUT_OFF"       => LiveCutOffInfo::try_from(value).map(|value| Self::LiveCutOff(value)),
            "WELCOME"       => WelcomeInfo::try_from(value).map(|value| Self::Welcome(value)),
            "WELCOME_GUARD" => WelcomeGuardInfo::try_from(value).map(|value| Self::WelcomeGuard(value)),
            "DANMU_MSG"     => DanmakuInfo::try_from(value).map(|value| Self::Danmaku(value)),
            "SEND_GIFT"     => SendGiftInfo::try_from(value).map(|value| Self::SendGift(value)),
            "SUPER_CHAT_MESSAGE" | "SUPER_CHAT_MESSAGE_JP"
                            => SuperChatInfo::try_from(value).map(|value| Self::SuperChat(value)),
            "INTERACT_WORD" => InteractInfo::try_from(value).map(|value| Self::Interact(value)),
            _ => { return Err(RawMessageDeserializeError::NotSupported(value.cmd)) },
        }
        .map_err(|_| RawMessageDeserializeError::DeserializeError)
    }
}