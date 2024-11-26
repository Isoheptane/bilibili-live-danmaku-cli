use chrono::{TimeDelta, Utc};
use client::LiveClient;
use colored::{ColoredString, Colorize};
use context::LiveContext;
use depack::DepackedMessage;
use message::data::GuardLevel;
use message::interact::InteractType;
use message::{LiveMessage, RawMessageDeserializeError};
use session_data::init_room_data;
use simple_logger::SimpleLogger;
use tungstenite::Message;
use std::thread::sleep;
use std::{env, time::Duration};

mod config;
mod context;
mod depack;
mod packet;
mod message;
mod session_data;
mod client;

use packet::{http::*, ws::*};
use config::Config;

use crate::session_data::SessionData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_level(log::LevelFilter::Info).env().with_timestamp_format(
        time::macros::format_description!("[hour]:[minute]:[second]")
    ).init().unwrap();
    // Get arguments
    let config = Config::from_args(env::args().collect());

    let (session, hosts) = match init_room_data(config.room_id, config.uid, config.sessdata.clone()) {
        Ok(result) => result,
        Err(e) => panic!("Failed to initialize room data: {}", e)
    };

    // Get host uri
    let host = hosts.get(0).expect("No available server in the list!").clone();
    let host_url = format!("wss://{}:{}/sub", host.host, host.wss_port);
    log::info!(
        target: "main",
        "Initializing connection to {} ...",
        host_url.bright_green()
    );
    
    loop {
        if let Err(e) = start_listening(session.clone(), &host_url, &config) {
            log::warn!(target: "init", "Error occured in the connection: \n {}", e.to_string());
        } else {
            log::warn!(target: "init", "Connection closed by server");
        }
        log::warn!(target: "init", "Reconnect after 5 seconds...");
        sleep(Duration::from_secs(5));
    }
}

fn start_listening(
    session: SessionData,
    host_url: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut context = LiveContext::new();

    let mut client = LiveClient::new(host_url, session)?;

    log::info!(
        target: "main",
        "Connected to live room"
    );

    let mut last_heartbeat = Utc::now();
    // Main loop
    loop {
        // Poll interval
        sleep(Duration::from_millis(config.poll_interval_ms));
        // Check heartbeat
        if last_heartbeat
            .checked_add_signed(TimeDelta::seconds(20))
            .is_some_and(|time| Utc::now() > time) 
        {
            let packet = heartbeat_packet_binary();
            if let Err(e) = client.send_message(Message::binary(packet)) {
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
            println!(
                " * {} 投餵了 {} 個 {}",
                info.user.username.bright_green(),
                info.gift_count.to_string().bright_yellow(),
                info.gift_name.bright_magenta(),
            );
            context.gift_list.remove(&info);
        }
        // Process messages
        let messages = match client.recv_messages() {
            Ok(x) => x,
            Err(e) => match e {
                client::ClientError::ConnectionClosed => {
                    return Ok(());
                }
                _ => {
                    log::debug!("Failed to poll messages");
                    return Err(e.into())
                }
            }
        };
        log::debug!("Ready to process depacked messages...");
        for message in messages {
            process_depacked_message(message, config, &mut context);
        }
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
            Err(RawMessageDeserializeError::DeserializeError(message)) => {
                log::warn!(target: "client", "Failed to deserialize raw message into live message");
                log::warn!(target: "client", "Live message: {}", message);
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
            let username = match (info.is_admin, info.user.guard_level) {
                (true, _) => info.user.username.bright_red(),
                (false, level) => get_colored_name(&info.user.username, level)
            };
            let badge_text = match info.user.medal {
                Some(medal) => {
                    format!("[{} {}] ", get_colored_badge_name(&medal.medal_name, medal.level), medal.level)
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
                context.gift_list.append_gift(
                    info, 
                    TimeDelta::milliseconds(config.gift_combo_interval_ms as i64), 
                    false
                );
            } else {
                println!(
                    " * {} 投餵了 {} 個 {}",
                    get_colored_name(&info.user.username, info.user.guard_level),
                    info.count.to_string().bright_yellow(),
                    info.gift_name.bright_magenta(),
                );
            }
        }
        LiveMessage::SuperChat(info) => {
            println!(
                "{} <{}> ({})\n : {}",
                "醒目留言".bright_cyan(),
                get_colored_name(&info.user.username, info.user.guard_level),
                format!("${:.2} {}s", info.price, info.keep_time).bright_yellow(),
                info.message.bright_yellow(),
            );
        }
        LiveMessage::Interact(info) => {
            let colored_name = get_colored_name(&info.user.username, info.user.guard_level);
            match info.interact_type {
                InteractType::Enter => println!(" * {} 進入了直播間", colored_name),
                InteractType::Follow => println!(" * {} 關注了你", colored_name),
                InteractType::Share => println!(" * {} 分.享了直播間", colored_name),
                InteractType::SpecialFollow => println!(" * {} 特別關注了你", colored_name),
                InteractType::MutualFollow => println!(" * {} 互關了你", colored_name),
            }
        }
        LiveMessage::GuardBuy(info) => {
            let guard_name = info.guard_level.name();
            println!(
                " * {} 成為了 {} ({} 個月)",
                get_colored_name(&info.user.username, Some(info.guard_level)),
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