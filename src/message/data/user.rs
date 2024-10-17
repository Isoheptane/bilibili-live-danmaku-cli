use super::{guard::GuardLevel, MedalInfo};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub uid: u64,
    pub username: String,
    pub guard_level: Option<GuardLevel>,
    pub medal: Option<MedalInfo>,
}