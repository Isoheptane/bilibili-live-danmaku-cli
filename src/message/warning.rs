use super::RawLiveMessage;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct WarningInfo {
    pub message: String
}

impl WarningInfo {
    pub fn try_from(value: RawLiveMessage) -> Option<Self> {
        Some(
            WarningInfo {
                message: value.msg?.to_string()
            }
        )
    }
}