use chrono::{TimeDelta, Utc};
use colored::{ColoredString, Colorize};
use context::LiveContext;
use depack::DepackedMessage;
use message::{LiveMessage, RawMessageDeserializeError};
use simple_logger::SimpleLogger;
use websocket::{ws::dataframe::DataFrame, Message, WebSocketError};
use std::{env, io::ErrorKind, thread::sleep, time::Duration};

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

fn start_listening(
    room_id: u64,
    uid: u64,
    token: &str,
    host_url: &str,
    config: &Config,
    context: &mut LiveContext
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
                println!(
                    " * {} 總共投餵了 {} 個 {}",
                    info.username.bright_green(),
                    info.gift_count.to_string().bright_yellow(),
                    info.gift_name.bright_magenta(),
                );
            } else {
                println!(
                    " * {} 投餵了 {} 個 {}",
                    info.username.bright_green(),
                    info.gift_count.to_string().bright_yellow(),
                    info.gift_name.bright_magenta(),
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
            },
            WebSocketError::NoDataAvailable => {
                // Server disconnect
                return Ok(());
            },
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
    context: &mut LiveContext
) {
    // Display certificate resp and heartbeat resp ony in debug
    let messages = match message {
        DepackedMessage::CertificateResp => {
            log::debug!(target: "client", "Received certificate response");
            return;
        },
        DepackedMessage::HeartbeatResp(count) => {
            log::debug!(target: "client", "Received heartbeat response ({})", count);
            return;
        },
        DepackedMessage::LiveMessages(messages) => messages
    };
    for raw_message in messages {
        let live_message = match LiveMessage::try_from(raw_message) {
            Ok(x) => x,
            Err(RawMessageDeserializeError::NotSupported(cmd)) => {
                log::debug!(target: "client", "Ignored unsupported command type {:#?}", cmd);
                continue;
            },
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
    context: &mut LiveContext
) {

    // Get colored name of a guard
    fn get_colored_name(name: &str, guard_level: Option<GuardLevel>) -> ColoredString {
        match guard_level {
            None => name.bright_green(),
            Some(GuardLevel::Captain) => name.bright_blue(),
            Some(GuardLevel::Commander) => name.bright_purple(),
            Some(GuardLevel::Governor) => name.bright_yellow(),
        }
    }

    // Get colored badge message
    fn get_colored_badge_name(name: &str, badge_level: u64) -> ColoredString {
        match badge_level {
            (1..=4)     => name.green(),
            (5..=8)     => name.blue(),
            (9..=12)    => name.magenta(),
            (13..=16)   => name.red(),
            (17..=20)   => name.yellow(),
            (21..=24)   => name.bright_green(),
            (25..=28)   => name.bright_blue(),
            (29..=32)   => name.bright_magenta(),
            (33..=36)   => name.bright_red(),
            (37..=40)   => name.bright_yellow(),
            _           => name.clear(),
        }
    }

    match message {
        LiveMessage::LiveStart(_) => {
            println!(" * {}", "直播開始了".bright_green());
        }
        LiveMessage::LiveStop(_) => {
            println!(" * {}", "直播結束了".bright_red());
        }
        LiveMessage::Welcome(info) => {
            let username = match info.is_admin {
                true => info.username.bright_red(),
                false => info.username.bright_green(),
            };
            println!(" * {} 進入了直播間", username);
        }
        LiveMessage::WelcomeGuard(info) => {
            println!(" * {} 進入了直播間", get_colored_name(&info.username, info.guard_level));
        }
        LiveMessage::Warning(info) => {
            println!(" * {} {}", "超管警告".bright_red(), info.message.bright_red())
        }
        LiveMessage::LiveCutOff(info) => {
            println!(" * {} {}", "直播被切斷".bright_red(), info.message.bright_red())
        }
        LiveMessage::Danmaku(info) => {
            let username = match (info.is_admin, info.guard_level) {
                (true, _) => info.username.bright_red(),
                (false, level) => get_colored_name(&info.username, level)
            };
            let badge_text = match info.badge {
                Some(badge) => {
                    format!("[{} {}] ", get_colored_badge_name(&badge.badge_name, badge.level), badge.level)
                }
                None => "".to_string()
            };
            println!(
                "{}{}\n : {}",
                badge_text,
                username,
                info.text
            );
        }
        LiveMessage::SendGift(info) => {
            if config.enable_gift_combo {
                // Only show notification when refresh is enabled
                if !context.gift_list.contains_info(&info) && config.gift_combo_refresh {
                    println!(
                        " * {} 投餵了 {}",
                        info.username.bright_green(),
                        info.gift_name.bright_magenta()
                    );
                }
                context.gift_list.append_gift(
                    info, 
                    TimeDelta::milliseconds(config.gift_combo_interval_ms as i64), 
                    config.gift_combo_refresh
                );
            } else {
                println!(
                    " * {} 投餵了 {} 個 {}",
                    info.username.bright_green(),
                    info.count.to_string().bright_yellow(),
                    info.gift_name.bright_magenta(),
                );
            }
        }
        LiveMessage::SuperChat(info) => {
            println!(
                "({}) <{}> {}",
                format!("$ {:.2}", info.price).bright_yellow(),
                info.username.bright_green(),
                info.message.bright_yellow(),
            )
        }
        LiveMessage::Interact(info) => {
            match info.interact_type {
                InteractType::Enter => {
                    println!(" * {} 進入了直播間", info.username.bright_green())
                }
                InteractType::Follow => {
                    println!(" * {} 關注了你", info.username.bright_green())
                }
                InteractType::Share => {
                    println!(" * {} 分享了直播間", info.username.bright_green())
                }
                InteractType::SpecialFollow => {
                    println!(" * {} 特別關注了你", info.username.bright_green())
                }
                InteractType::MutualFollow => {
                    println!(" * {} 互關了你", info.username.bright_green())
                }
            }
        }
        LiveMessage::GuardBuy(info) => {
            let guard_name = match info.guard_level {
                GuardLevel::Captain => "艦長",
                GuardLevel::Commander => "提督",
                GuardLevel::Governor => "總督",
            };
            println!(
                " * {} 成為了 {} ({} 個月)",
                get_colored_name(&info.username, Some(info.guard_level)),
                get_colored_name(guard_name, Some(info.guard_level)),
                info.count.to_string().bright_yellow()
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