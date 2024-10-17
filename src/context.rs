use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

use crate::message::{data::UserInfo, gift::SendGiftInfo, super_chat::SuperChatInfo};

#[derive(Debug, Clone)]
pub struct CombinedSendGiftInfo {
    pub user: UserInfo,
    pub gift_name: String,
    pub gift_count: u64,
    pub event_count: u64,
    pub expiry_time: DateTime<Utc>
}

pub struct SendGiftList {
    gifts: HashMap<(u64, String), CombinedSendGiftInfo>,
}

#[allow(unused)]
impl SendGiftList {
    pub fn contains_info(&self, info: &SendGiftInfo) -> bool{
        self.gifts.contains_key(&(info.user.uid, info.gift_name.clone()))
    }
    pub fn append_gift(&mut self, info: SendGiftInfo, expire_interval: TimeDelta, refresh_time: bool) {
        let key = (info.user.uid, info.gift_name.clone());
        let combined_info = self.gifts.entry(key).or_insert(
            CombinedSendGiftInfo { 
                user: info.user,
                gift_name: info.gift_name.clone(),
                gift_count: 0,
                event_count: 0,
                expiry_time: Utc::now().checked_add_signed(expire_interval).expect("Failed to update time")
            }
        );
        combined_info.gift_count += info.count;
        combined_info.event_count += 1;
        if refresh_time {
            combined_info.expiry_time = Utc::now().checked_add_signed(expire_interval).expect("Failed to update time")
        }
    }
    pub fn get_expired(&self) -> Vec<CombinedSendGiftInfo> {
        self.gifts.iter().filter(|(_, combined_info)| {
            Utc::now() > combined_info.expiry_time
        })
        .map(|(_, combined_info)| combined_info.clone())
        .collect()
    }
    pub fn remove(&mut self, info: &CombinedSendGiftInfo) {
        self.gifts.remove(&(info.user.uid, info.gift_name.clone()));
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SuperChatPresistent {
    pub superchat_info: SuperChatInfo,
    pub expiry_time: DateTime<Utc>,
    pub next_show_time: DateTime<Utc>,
}

#[allow(unused)]
pub struct SuperChatList {
    superchats: HashMap<u64, SuperChatPresistent>,
}

#[allow(unused)]
impl SuperChatList {
    pub fn append_superchat(&mut self, info: SuperChatInfo, show_interval: TimeDelta) {
        let now = Utc::now();
        let expiry_time = now.checked_add_signed(TimeDelta::seconds(info.keep_time as i64)).expect("Failed to update time");
        let next_show_time = now.checked_add_signed(TimeDelta::seconds(info.keep_time as i64)).expect("Failed to update time");
        let uid = info.user.uid;
        let presistent = SuperChatPresistent {
            superchat_info: info,
            expiry_time,
            next_show_time
        };
        self.superchats.insert(uid, presistent);
    }
    /// Return expired and net show time
    pub fn get_should_show(&self) -> Vec<SuperChatPresistent> {
        self.superchats.iter().filter(|(_, presistent)| {
            Utc::now() > presistent.expiry_time || Utc::now() > presistent.next_show_time
        })
        .map(|(_, presistent)| presistent.clone())
        .collect()
    }
    /// Remove expired and step up next_show_time
    pub fn update_step(&mut self, show_interval: TimeDelta) {
        self.superchats.retain(|_, presistent| presistent.expiry_time > Utc::now());
        for (_, sc) in self.superchats.iter_mut() {
            if Utc::now() > sc.next_show_time {
                sc.next_show_time = sc.next_show_time.checked_add_signed(show_interval).expect("Failed to update time");
            }
        }
    }
}

#[allow(unused)]
pub struct LiveContext {
    pub gift_list: SendGiftList,
    pub superchat_list: SuperChatList
}

impl LiveContext {
    pub fn new() -> LiveContext {
        LiveContext {
            gift_list: SendGiftList {
                gifts: HashMap::new()
            },
            superchat_list: SuperChatList { 
                superchats: HashMap::new()
            }
        }
    }
}