use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

use crate::message::gift::SendGiftInfo;

#[derive(Debug, Clone)]
pub struct CombinedSendGiftInfo {
    pub user_id: u64,
    pub username: String,
    pub gift_name: String,
    pub gift_count: u64,
    pub event_count: u64,
    pub expire_time: DateTime<Utc>
}

pub struct SendGiftList {
    gifts: HashMap<(u64, String), CombinedSendGiftInfo>,
}

impl SendGiftList {
    pub fn contains_info(&self, info: &SendGiftInfo) -> bool{
        self.gifts.contains_key(&(info.user_id, info.gift_name.clone()))
    }
    pub fn append_gift(&mut self, info: SendGiftInfo, expire_interval: TimeDelta, refresh_time: bool) {
        let key = (info.user_id, info.gift_name.clone());
        let combined_info = self.gifts.entry(key).or_insert(
            CombinedSendGiftInfo { 
                user_id: info.user_id,
                username: info.username.clone(),
                gift_name: info.gift_name.clone(),
                gift_count: 0,
                event_count: 0,
                expire_time: Utc::now().checked_add_signed(expire_interval).expect("Failed to update time")
            }
        );
        combined_info.gift_count += info.count;
        combined_info.event_count += 1;
        if refresh_time {
            combined_info.expire_time = Utc::now().checked_add_signed(expire_interval).expect("Failed to update time")
        }
    }
    pub fn get_expired(&self) -> Vec<CombinedSendGiftInfo> {
        self.gifts.iter().filter(|(_, combined_info)| {
            Utc::now() > combined_info.expire_time
        })
        .map(|(_, combined_info)| combined_info.clone())
        .collect()
    }
    pub fn remove(&mut self, info: &CombinedSendGiftInfo) {
        self.gifts.remove(&(info.user_id, info.gift_name.clone()));
    }
}

pub struct LiveContext {
    pub gift_list: SendGiftList
}

impl LiveContext {
    pub fn new() -> LiveContext {
        LiveContext {
            gift_list: SendGiftList {
                gifts: HashMap::new()
            }
        }
    }
}