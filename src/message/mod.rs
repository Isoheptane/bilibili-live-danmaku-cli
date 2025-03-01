pub mod danmaku;
pub mod gift;
pub mod guard;
pub mod interact;
pub mod live;
pub mod super_chat;
pub mod warning;
pub mod welcome;
pub mod data;

use std::fmt::Display;

use derive_more::Display;
use serde::Deserialize;
use serde_json::Value;

use danmaku::DanmakuInfo;
use gift::SendGiftInfo;
use guard::GuardBuyInfo;
use interact::InteractInfo;
use live::{LiveStartInfo, LiveStopInfo, LiveCutOffInfo};
use super_chat::SuperChatInfo;
use warning::WarningInfo;
use welcome::{WelcomeInfo, WelcomeGuardInfo};

use self::gift::GiftTopInfo;

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
    pub data: Option<Value>
}

impl Display for RawLiveMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Room ID: {:?}\n", self.room_id)?;
        write!(f, "Command: {}\n", self.cmd)?;
        write!(f, "Message: {:?}\n", self.msg)?;
        write!(f, "Info: {:#?}\n", self.info)?;
        write!(f, "Data: {:#?}\n", self.data)
    }
}

#[derive(Debug, Display)]
pub enum RawMessageDeserializeError {
    NotSupported(String),
    DeserializeError(RawLiveMessage)
}

impl std::error::Error for RawMessageDeserializeError {
    fn description(&self) -> &str {
        match self {
            Self::NotSupported(_) => "Message type not supported",
            Self::DeserializeError(_) => "Failed to deserialze message"
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
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
    GuardBuy        (GuardBuyInfo),
    GiftTop         (GiftTopInfo),
}

impl TryFrom<RawLiveMessage> for LiveMessage {
    type Error = RawMessageDeserializeError;

    fn try_from(value: RawLiveMessage) -> Result<Self, Self::Error> {
        let value_backup = value.clone();
        match value.cmd.as_str() {
            "LIVE"          
                => LiveStartInfo::      try_from(value).map(|value| Self::LiveStart(value)),
            "PREPARING"     
                => LiveStopInfo::       try_from(value).map(|value| Self::LiveStop(value)),
            "WARNING"       
                => WarningInfo::        try_from(value).map(|value| Self::Warning(value)),
            "CUT_OFF"       
                => LiveCutOffInfo::     try_from(value).map(|value| Self::LiveCutOff(value)),
            "WELCOME"       
                => WelcomeInfo::        try_from(value).map(|value| Self::Welcome(value)),
            "WELCOME_GUARD" 
                => WelcomeGuardInfo::   try_from(value).map(|value| Self::WelcomeGuard(value)),
            "DANMU_MSG"     
                => DanmakuInfo::        try_from(value).map(|value| Self::Danmaku(value)),
            "SEND_GIFT"     
                => SendGiftInfo::       try_from(value).map(|value| Self::SendGift(value)),
            "SUPER_CHAT_MESSAGE" | "SUPER_CHAT_MESSAGE_JP"
                => SuperChatInfo::      try_from(value).map(|value| Self::SuperChat(value)),
            "INTERACT_WORD" 
                => InteractInfo::       try_from(value).map(|value| Self::Interact(value)),
            "GUARD_BUY"     
                => GuardBuyInfo::       try_from(value).map(|value| Self::GuardBuy(value)),
            "GIFT_TOP"      
                => GiftTopInfo::        try_from(value).map(|value| Self::GiftTop(value)),
            _ => { return Err(RawMessageDeserializeError::NotSupported(value.cmd)) },
        }
        .ok_or(RawMessageDeserializeError::DeserializeError(value_backup))
    }
}