use chrono::{TimeDelta, Utc};
use colored::Colorize;
use message::{LiveMessage, RawLiveMessage};
use serde::{Deserialize, Serializer};
use simple_logger::SimpleLogger;
use tungstenite::Message;
use std::{env, io::Read, thread::sleep, time::Duration};
use tokio;

mod config;
mod packet;
mod message;

use packet::{http::*, ws::*};
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_level(log::LevelFilter::Info).env().with_timestamp_format(
        time::macros::format_description!("[hour]:[minute]:[second]")
    ).init().unwrap();
    // Get arguments
    let config = Config::from_args(env::args().collect());

    // Start calling APIs
    let room_data: RoomInitData = reqwest::get(format!(
        "https://api.live.bilibili.com/room/v1/Room/room_init?id={}",
        config.room_id
    )).await?
        .json::<HttpAPIResponse<RoomInitData>>().await?
        .response_data()
        .expect("Invalid room_init response data.");
    
    let room_id = room_data.room_id;

    log::info!(
        target: "main",
        "Requested real room ID: {}", room_id.to_string().bright_green()
    );
let danmaku_info_data: DanmakuInfoData = reqwest::Client::new()
    .get(format!(
        "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}",
        room_id
    ))
    .header(
        reqwest::header::COOKIE, 
        config.sessdata.map(
            |sessdata| format!("SESSDATA={}", sessdata)
        ).unwrap_or("".to_string()))
    .send().await?
        .json::<HttpAPIResponse<DanmakuInfoData>>().await?
        .response_data()
        .expect("Invalid danmaku_info response data.");
    log::info!(
        target: "main",
        "Requested token and WebSocket servers. {} servers available.",
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
    let host_url = url::Url::parse(&host_url).expect("Failed to parse URL");
    
    loop {
        if let Err(e) = start_listening(room_id, config.uid.unwrap_or(0), &token, &host_url).await {
            log::warn!(target: "init", "Connection closed! \n {}", e.to_string());
            log::warn!(target: "init", "Trying to reconnect after 5 seconds.");
            sleep(Duration::from_secs(5));
        }
    }
}

async fn start_listening(
    room_id: u64,
    uid: u64,
    token: &String,
    host_url: &url::Url
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut stream, _) = tungstenite::connect(host_url)?;
    log::info!(
        target: "client",
        "Successfully connected to server"
    );
    let mut last_heartbeat = Utc::now();
    // Send certificate
    stream.send(Message::binary(create_certificate_packet(uid, room_id, token)?))?;
    // Main loop
    loop {
        sleep(Duration::from_millis(10));
        // Check heartbeat
        if last_heartbeat
            .checked_add_signed(TimeDelta::seconds(20))
            .is_some_and(|time| Utc::now() > time) 
        {
            let packet = create_heartbeat_packet();
            if let Ok(packet) = packet {
                match stream.send(Message::binary(packet)) {
                    Ok(_) => {
                        last_heartbeat = Utc::now();
                        log::debug!(
                            target: "client",
                            "Sent heartbeat packet"
                        );
                    },
                    Err(e) => {
                        log::warn!(
                            target: "client",
                            "Failed to send heartbeat packet:\n {}",
                            e
                        );
                    }
                }
            }
        }
        // Read all packets
        while stream.can_read() {
            let msg = match stream.read() {
                Ok(msg) => msg,
                Err(e) => match e {
                    tungstenite::Error::ConnectionClosed => { return Err(e.into()); },
                    _ => {
                        log::warn!(
                            target: "client", 
                            "Failed to receive message: {}", 
                            e
                        );
                        continue;
                    }
                }
            };
            let data = msg.into_data();
            let (header, body) = match deserialize_packet(data.as_slice()) {
                Ok(x) => x,
                Err(_) => { continue; }
            };
            log::debug!(
                target: "client", 
                "Received packet: {:?}",
                header
            );
            process_packet(header, body);
        }
    }
}

#[derive(Debug)]
pub enum PacketProcessError {
    DecompressError,
    PacketDeserializeError,
    DeserializeError(Option<Box<dyn std::error::Error>>)
}

impl std::error::Error for PacketProcessError {
    fn description(&self) -> &str {
        match self {
            Self::DecompressError => "Failed to decompress data",
            Self::PacketDeserializeError => "Failed to deserialize packet header and body",
            Self::DeserializeError(_) => "Failed to deserialize packet body",
        }
    }
}

impl std::fmt::Display for PacketProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PacketProcessError {{ type: ")?;
        match self {
            Self::DecompressError => f.write_str("DecompressionError")?,
            Self::PacketDeserializeError => f.write_str("PacketDeserializeError")?,
            Self::DeserializeError(e) => f.write_str(
                format!("DeserializeError, innerError: {:?}", e).as_str()
            )?
        };
        write!(f, "}}")
    }
}

fn process_packet(header: PacketHeader, body: &[u8]) -> Result<(), PacketProcessError> {
    if header.protocol == Protocol::CommandBrotli as u16 {
        let mut data: Vec<u8> = vec![];
        brotli::Decompressor::new(body, 4096)
            .read_to_end(&mut data)
            .map_err(|e| PacketProcessError::DecompressError)?;
        let total_length: usize = data.len();
        let mut read_len: usize = 0;
        while read_len < total_length {
            let (header, body) = match deserialize_packet(&data) {
                Ok(x) => x,
                Err(_) => {
                    log::debug!(
                        target: "client",
                        "Error occured while deserializing packet: {}",
                        PacketProcessError::PacketDeserializeError
                    );
                    continue;
                }
            };
            read_len += body.len() + 16;
            match process_packet(header, body) {
                Ok(()) => {},
                Err(e) => {
                    log::debug!(
                        target: "client",
                        "Error occured while processing inner packet: {}",
                        e
                    )
                }
            }
            log::debug!(
                target: "client", 
                "Processed inner message block. Read length {}/{} bytes.",
                read_len,
                total_length
            )
        }
        return Ok(());
    } else if header.protocol == Protocol::CommandZlib as u16 {
        todo!();
        return Ok(());
    }
    // Raw packet process
    // Heartbeat 
    if header.packet_type != PacketType::Command as u32 {
        //  TODO
        return Ok(());
    }
    // Add ending zero to make sure String::from_utf8 will work
    let mut bytes = body.to_vec();
    bytes.push(0);
    let json = String::from_utf8(bytes)
        .map_err(|e| PacketProcessError::DeserializeError(Some(e.into())))?;
    log::debug!(target: "client", "Processing JSON string: {:#?}", json);
    let raw_live_message: RawLiveMessage = serde_json::from_str(&json)
        .map_err(|e| PacketProcessError::DeserializeError(Some(e.into())))?;
    let message = LiveMessage::try_from(raw_live_message)
        .map_err(|_| PacketProcessError::DeserializeError(None))?;
    log::info!(target: "client", "Message: {:#?}", message);   

    Ok(())
}