use serde::Deserialize;



#[derive(Debug, Deserialize, Clone, Copy)]
pub enum LiveMessageType {
    #[serde(rename = "LIVE")]
    LiveStart,
    #[serde(rename = "PREPARING")]
    LiveEnd,
    #[serde(rename = "DANMU_MSG")]
    Danmaku,
    #[serde(rename = "SEND_GIFT")]
    SendGift,
    #[serde(rename = "WELCOME")]
    Welcome,
    #[serde(rename = "WELCOME_GUARD")]
    WelcomeGuard,
}

pub struct LiveMessage {

}