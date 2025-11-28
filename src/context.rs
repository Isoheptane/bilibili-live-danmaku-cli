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

impl CombinedSendGiftInfo {
    pub fn expired(&self) -> bool {
        return Utc::now() > self.expiry_time
    }
}

pub struct SendGiftList {
    gifts: HashMap<(u64, String), CombinedSendGiftInfo>,
}

impl SendGiftList {
    #[allow(unused)]
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
                expiry_time: Utc::now().checked_add_signed(expire_interval)
                    .expect("Failed to update time: Time out of range")
            }
        );
        combined_info.gift_count += info.count;
        combined_info.event_count += 1;
        if refresh_time {
            combined_info.expiry_time = Utc::now().checked_add_signed(expire_interval)
                .expect("Failed to update time: Time out of range")
        }
    }
    /// Return expired info, this will remove expired info from the pending list
    pub fn get_expired(&mut self) -> Vec<CombinedSendGiftInfo> {
        let expired_list: Vec<CombinedSendGiftInfo> = self.gifts.iter()
            .filter(|(_, info)| info.expired())
            .map(|(_, combined_info)| combined_info.clone())
            .collect();
        for expired_info in expired_list.iter() {
            self.remove(expired_info);
        }
        return expired_list;
    }
    pub fn remove(&mut self, info: &CombinedSendGiftInfo) {
        self.gifts.remove(&(info.user.uid, info.gift_name.clone()));
    }
}

#[derive(Debug, Clone)]
pub struct SuperChatPresistent {
    pub superchat_info: SuperChatInfo,
    pub send_time: DateTime<Utc>,
    pub next_show_time: DateTime<Utc>,
}

impl SuperChatPresistent {
    pub fn expiry_time(&self) -> DateTime<Utc> {
        self.send_time.checked_add_signed(TimeDelta::seconds(self.superchat_info.keep_time as i64))
            .expect("Failed to calculate time: Time out of range")
    }
    pub fn expired(&self) -> bool {
        return Utc::now() > self.expiry_time()
    }
    pub fn should_show(&self) -> bool {
        return Utc::now() > self.next_show_time
    }
}

pub struct SuperChatList {
    superchats: HashMap<(u64, DateTime<Utc>), SuperChatPresistent>,
}

impl SuperChatList {

    pub fn append_superchat(&mut self, info: SuperChatInfo, show_interval: TimeDelta) {
        let uid = info.user.uid;
        let send_time = Utc::now();
        let next_show_time = send_time.checked_add_signed(show_interval).expect("Failed to update time");
        let presistent = SuperChatPresistent {
            superchat_info: info,
            send_time,
            next_show_time
        };
        self.superchats.insert((uid, send_time), presistent);
    }
    /// Return superchats that need to show again & expired superchats. This will remove expired superchats.
    pub fn get_should_show(&mut self) -> Vec<SuperChatPresistent> {
        let should_show_list: Vec<SuperChatPresistent> = self.superchats.iter()
            .filter(|(_, sc)| sc.expired() | sc.should_show())
            .map(|(_, presistent)| presistent.clone())
            .collect();
        // Remove expired in the should show list
        for info in should_show_list.iter() {
            if info.expired() {
                self.superchats.remove(&(info.superchat_info.user.uid, info.send_time));
            }
        }
        return should_show_list;
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