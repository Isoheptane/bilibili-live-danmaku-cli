use chrono::{TimeDelta, Utc};
use colored::{ColoredString, Colorize};
use context::LiveContext;
use depack::DepackedMessage;
use message::{LiveMessage, RawMessageDeserializeError};
use simple_logger::SimpleLogger;
use websocket::{ws::dataframe::DataFrame, Message, WebSocketError};
use std::{env, io::ErrorKind, thread::sleep, time::Duration};
use reqwest::blocking as req;
use serde::de::Unexpected::Str;

mod config;
mod context;
mod depack;
mod packet;
mod message;

use packet::{http::*, ws::*};
use config::Config;

use crate::{depack::depack_packets, message::{guard::GuardLevel, interact::InteractType}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_level(log::LevelFilter::Info).env().with_timestamp_format(
        time::macros::format_description!("[hour]:[minute]:[second]")
    ).init().unwrap();
    // Get arguments
    let config = Config::from_args(env::args().collect());

    // Start calling APIs
    let agent = ureq::builder().tls_connector(native_tls::TlsConnector::new().unwrap().into()).build();
    // Get room data for the real room id
    let room_data: RoomInitData = agent.get(
        &format!("https://api.live.bilibili.com/room/v1/Room/room_init?id={}", config.room_id)
    )
        .call()
        .expect("Failed to request for room_init data")
        .into_json::<HttpAPIResponse<RoomInitData>>()
        .expect("Failed to parse room_init json data")
        .response_data()
        .expect("Response data is empty");

    let room_id = room_data.room_id;
    log::info!(
        target: "main",
        "Requested real room ID: {}", room_id.to_string().bright_green()
    );
    // Get danmaku info data
    let danmaku_info_data: DanmakuInfoData = agent.get(
        &format!("https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}", room_id)
    )
        .set("Cookie", format!("SESSDATA={}", config.sessdata.clone().unwrap_or_default()).as_str())
        .call()
        .expect("Failed to request for room_init data")
        .into_json::<HttpAPIResponse<DanmakuInfoData>>()
        .expect("Failed to parse danmaku_info json data")
        .response_data()
        .expect("Response data is empty");

    log::info!(
        target: "main",
        "Requested token and WebSocket servers. {} servers available",
        danmaku_info_data.host_list.len().to_string().bright_green()
    );

    // Get token and host uri
    let token = danmaku_info_data.token;
    let host = danmaku_info_data.host_list.get(0).expect("No available server in the list!").clone();
    let host_url = format!("wss://{}:{}/sub", host.host, host.wss_port);
    log::info!(
        target: "main",
        "Initializing connection to {} ...",
        host_url.bright_green()
    );

    let mut context = LiveContext::new();

    loop {
        if let Err(e) = start_listening(room_id, config.uid.unwrap_or(0), &token, &host_url, &config, &mut context) {
            log::warn!(target: "init", "Error occured in the connection: \n {}", e.to_string());
        } else {
            log::warn!(target: "init", "Connection closed by server");
        }
        log::warn!(target: "init", "Reconnect after 5 seconds...");
        sleep(Duration::from_secs(5));
    }
}

fn send_message(config: &Config, message: String) -> req::Response {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", config.bot_token);
    let response = req::Client::new()
        .post(&url)
        .form(&[
            ("chat_id", config.chat_id.to_string().as_str()),
            ("text", message.as_str()),
            ("parse_mode", "HTML")
        ])
        .send()
        .expect("Failed to send message to Telegram");
    return response;
}

fn start_listening(
    room_id: u64,
    uid: u64,
    token: &str,
    host_url: &str,
    config: &Config,
    context: &mut LiveContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = websocket::ClientBuilder::new(host_url).unwrap()
        .connect_secure(None).unwrap();
    // Client should work in nonblocking mode
    client.set_nonblocking(true)?;
    log::info!(target: "client", "Successfully connected to server");

    let mut last_heartbeat = Utc::now();
    // Send certificate
    client.send_message(&Message::binary(certificate_packet(uid, room_id, token)?))?;
    log::debug!(target: "client", "Certificate packet sent");

    // Main loop
    'main: loop {
        // Poll interval
        sleep(Duration::from_millis(config.poll_interval_ms));
        // Check heartbeat
        if last_heartbeat
            .checked_add_signed(TimeDelta::seconds(20))
            .is_some_and(|time| Utc::now() > time)
        {
            let packet = heartbeat_packet();
            if let Err(e) = client.send_message(&Message::binary(packet)) {
                log::warn!(
                    target: "client",
                    "Failed to send heartbeat packet:\n {}",
                    e
                );
            } else {
                last_heartbeat = Utc::now();
                log::debug!(
                    target: "client",
                    "Heartbeat packet sent"
                );
            }
        }
        // Check context events
        for info in context.gift_list.get_expired() {
            if config.gift_combo_refresh && info.event_count > 1 {
                send_message(
                    config,
                    format!(
                        " * <b>{}</b> 总共投喂了 <b>{}</b> 个 <b>{}</b>",
                        info.username,
                        info.gift_count,
                        info.gift_name
                    ),
                );
            } else {
                send_message(
                    config,
                    format!(
                        " * <b>{}</b> 投喂了 <b>{}</b> 个 <b>{}</b>",
                        info.username,
                        info.gift_count,
                        info.gift_name
                    ),
                );
            }
            context.gift_list.remove(&info);
        }

        // Read all packets
        let error = 'poll: loop {
            let msg = match client.recv_message() {
                Ok(x) => x,
                Err(e) => break 'poll e
            };
            if msg.is_close() {
                return Ok(());
            }
            let data = msg.take_payload();
            let (header, body) = match deserialize_packet(data.as_slice()) {
                Ok(x) => x,
                Err(_) => { continue; }
            };
            log::trace!(
                target: "client",
                "Received packet: {:?}",
                header
            );
            let message = match depack_packets(header, body) {
                Ok(message) => message,
                Err(e) => {
                    log::debug!(target: "client", "Failed to depack packets: {}", e);
                    continue 'poll;
                }
            };
            process_depacked_message(message, config, context);
        };
        // Fetch out websocket errors
        let error = match error {
            WebSocketError::IoError(io_error) => {
                // Continue main loop on blocking operations
                if io_error.kind() == ErrorKind::WouldBlock {
                    continue 'main;
                } else {
                    WebSocketError::IoError(io_error)
                }
            }
            WebSocketError::NoDataAvailable => {
                // Server disconnect
                return Ok(());
            }
            e => e
        };
        log::warn!(
            target: "client",
            "Error occured when trying to poll message from WebSocet: {}",
            error
        )
    }
}

fn process_depacked_message(
    message: DepackedMessage,
    config: &Config,
    context: &mut LiveContext,
) {
    // Display certificate resp and heartbeat resp ony in debug
    let messages = match message {
        DepackedMessage::CertificateResp => {
            log::debug!(target: "client", "Received certificate response");
            return;
        }
        DepackedMessage::HeartbeatResp(count) => {
            log::debug!(target: "client", "Received heartbeat response ({})", count);
            return;
        }
        DepackedMessage::LiveMessages(messages) => messages
    };
    for raw_message in messages {
        let live_message = match LiveMessage::try_from(raw_message) {
            Ok(x) => x,
            Err(RawMessageDeserializeError::NotSupported(cmd)) => {
                log::debug!(target: "client", "Ignored unsupported command type {:#?}", cmd);
                continue;
            }
            Err(RawMessageDeserializeError::DeserializeError) => {
                log::debug!(target: "client", "Failed to deserialize raw message into live message");
                continue;
            }
        };
        process_live_message(live_message, config, context);
    }
}

fn process_live_message(
    message: LiveMessage,
    config: &Config,
    context: &mut LiveContext,
) {
    // Get leveled name of a guard
    fn get_leveled_name(name: &str, guard_level: Option<GuardLevel>) -> String {
        match guard_level {
            None => format!("{}", name),
            Some(GuardLevel::Captain) => format!("🛳🛳️ {} (舰长)", name),
            Some(GuardLevel::Commander) => format!("⛴️ {} (提督)", name),
            Some(GuardLevel::Governor) => format!("⛴️ {} (总督)", name),
        }
    }

    // Get leveled badge message
    fn get_leveled_badge_name(name: &str, badge_level: u64) -> String {
        match badge_level {
            (1..=4) => format!("🟢 {}", name),
            (5..=8) => format!("🔵 {}", name),
            (9..=12) => format!("🔵 {}", name),
            (13..=16) => format!("🔵 {}", name),
            (17..=20) => format!("🟡 {}", name),
            (21..=24) => format!("🟩 {}", name),
            (25..=28) => format!("🟦 {}", name),
            (29..=32) => format!("🟪 {}", name),
            (33..=36) => format!("🟥 {}", name),
            (37..=40) => format!("🟨 {}", name),
            _ => String::from(name)
        }
    }

    match message {
        LiveMessage::LiveStart(_) => {
            send_message(
                config,
                format!(
                    " * <i>直播开始了，可以前往<a href=\"https://live.bilibili.com/{}\">直播间</a>观看喵</i>",
                    config.room_id
                ),
            );
        }
        LiveMessage::LiveStop(_) => {
            send_message(config, " * <i>直播结束了，再见喵</i>".to_string());
        }
        LiveMessage::Welcome(info) => {
            let username = match info.is_admin {
                true => info.username.bright_red(),
                false => info.username.bright_green(),
            };
            send_message(
                config,
                format!(" * <i>{} 进入了直播间</i>", username),
            );
        }
        LiveMessage::WelcomeGuard(info) => {
            send_message(
                config,
                format!(
                    " * <i>{} 进入了直播间</i>",
                    get_leveled_name(&info.username, info.guard_level)
                ),
            );
        }
        LiveMessage::Warning(info) => {
            send_message(
                config,
                format!(" * <b>超管警告: {}</b>", info.message),
            );
        }
        LiveMessage::LiveCutOff(info) => {
            send_message(
                config,
                format!(" * <b>直播被切断: {}</b>", info.message),
            );
        }
        LiveMessage::Danmaku(info) => {
            let username = match (info.is_admin, info.guard_level) {
                (true, _) => format!("[房管] {}", info.username),
                (false, level) => get_leveled_name(&info.username, level)
            };
            let badge_text = match info.badge {
                Some(badge) => {
                    format!("[{} {}] ", get_leveled_badge_name(&badge.badge_name, badge.level), badge.level)
                }
                None => "".to_string()
            };
            send_message(
                config,
                format!(
                    "<b>{} {}</b>: {}",
                    badge_text,
                    username,
                    info.text
                ),
            );
        }
        LiveMessage::SendGift(info) => {
            if config.enable_gift_combo {
                // Only show notification when refresh is enabled
                if !context.gift_list.contains_info(&info) && config.gift_combo_refresh {
                    send_message(
                        config,
                        format!(
                            " * <b>{}</b> 投喂了 <b>{}</b>",
                            info.username,
                            info.gift_name
                        ),
                    );
                }
                context.gift_list.append_gift(
                    info,
                    TimeDelta::milliseconds(config.gift_combo_interval_ms as i64),
                    config.gift_combo_refresh,
                );
            } else {
                send_message(
                    config,
                    format!(
                        " * <b>{}</b> 投喂了 <b>{}</b> 个 <b>{}</b>",
                        info.username,
                        info.count,
                        info.gift_name
                    ),
                );
            }
        }
        LiveMessage::SuperChat(info) => {
            send_message(
                config,
                format!(
                    "[SC {}] <b>{}</b>: {}",
                    format!("$ {:.2}", info.price),
                    info.username,
                    info.message
                ),
            );
        }
        LiveMessage::Interact(info) => {
            match info.interact_type {
                InteractType::Enter => {
                    send_message(
                        config,
                        format!("<i> * <b>{}</b> 进入了直播间<i>", info.username),
                    );
                }
                InteractType::Follow => {
                    send_message(
                        config,
                        format!("<i> * <b>{}</b> 关注了主播<i>", info.username),
                    );
                }
                InteractType::Share => {
                    send_message(
                        config,
                        format!("<i> * <b>{}</b> 分享了直播间<i>", info.username),
                    );
                }
                InteractType::SpecialFollow => {
                    send_message(
                        config,
                        format!("<i> * <b>{}</b> 特别关注了主播<i>", info.username),
                    );
                }
                InteractType::MutualFollow => {
                    send_message(
                        config,
                        format!("<i> * <b>{}</b> 互关了主播<i>", info.username),
                    );
                }
            }
        }
        LiveMessage::GuardBuy(info) => {
            let guard_name = match info.guard_level {
                GuardLevel::Captain => "舰长",
                GuardLevel::Commander => "提督",
                GuardLevel::Governor => "总督",
            };
            send_message(
                config,
                format!(
                    " * <b>{}</b> 成为了 <b>{}</b> ({} 个月)",
                    get_leveled_name(&info.username, Some(info.guard_level)),
                    get_leveled_name(guard_name, Some(info.guard_level)),
                    info.count
                ),
            );
        }
        #[allow(unreachable_patterns)]
        other => {
            log::debug!(
                target: "client",
                "Ignored message that doesn't support display: {:#?}",
                other
            )
        }
    }
}
