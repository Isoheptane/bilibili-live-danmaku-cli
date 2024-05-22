use chrono::{TimeDelta, Utc};
use colored::Colorize;
use message::{LiveMessage, RawLiveMessage};
use simple_logger::SimpleLogger;
use websocket::{ws::dataframe::DataFrame, Message, WebSocketError};
use std::{env, io::{ErrorKind, Read}, thread::sleep, time::Duration};

mod config;
mod packet;
mod message;

use packet::{http::*, ws::*};
use config::Config;

use crate::message::RawMessageDeserializeError;

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
        .set("Cookie", format!("SESSDATA={}", config.sessdata.unwrap_or_default()).as_str())
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
    
    loop {
        if let Err(e) = start_listening(room_id, config.uid.unwrap_or(0), &token, &host_url) {
            log::warn!(target: "init", "Error occured in the connection: \n {}", e.to_string());
        } else {
            log::warn!(target: "init", "Connection closed by server");
        }
        log::warn!(target: "init", "Trying to reconnect after 5 seconds");
        sleep(Duration::from_secs(5));
    }
}

fn start_listening(
    room_id: u64,
    uid: u64,
    token: &str,
    host_url: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // *req.version_mut() = http::Version::HTTP_11;
    let mut client = websocket::ClientBuilder::new(host_url).unwrap().connect_secure(None).unwrap();
    // Client should always work in nonblocking mode
    client.set_nonblocking(true)?;
    log::info!(target: "client", "Successfully connected to server");

    let mut last_heartbeat = Utc::now();
    // Send certificate
    client.send_message(&Message::binary(certificate_packet(uid, room_id, token)?))?;
    log::debug!(target: "client", "Certificate packet sent");
    // Main loop

    loop {
        sleep(Duration::from_millis(100));
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
        // Read all packets
        // log::trace!(target: "client", "Begin receiving message...");
        loop {
            let msg = match client.recv_message() {
                Ok(x) => x,
                Err(e) => match e {
                    WebSocketError::IoError(io_error) => {
                        if io_error.kind() == ErrorKind::WouldBlock {
                            // Jump out of poll cycle if would block
                            break;
                        } else {
                            // Other IO error
                            return Err(WebSocketError::IoError(io_error).into());
                        }
                    },
                    WebSocketError::NoDataAvailable => {
                        // Server disconnect
                        return Ok(());
                    },
                    e => {
                        log::debug!(
                            target: "client", 
                            "Error occured while trying to receive message: {:?}",
                            e
                        );
                        break;
                    }
                }
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
            .map_err(|_| PacketProcessError::DecompressError)?;
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
            log::trace!(
                target: "client", 
                "Processed inner message block. Read length {}/{} bytes.",
                read_len,
                total_length
            )
        }
        return Ok(());
    } else if header.protocol == Protocol::CommandZlib as u16 {
        todo!();
    }
    // Raw packet process
    // Heartbeat 
    if header.packet_type != PacketType::Command as u32 {
        //  TODO
        return Ok(());
    }
    // Add ending zero to make sure String::from_utf8 will work
    let json = String::from_utf8(body.to_vec())
        .map_err(|e| PacketProcessError::DeserializeError(Some(e.into())))?;
    log::trace!(target: "client", "Processing JSON string: {:#?}", json);
    let raw_live_message: RawLiveMessage = serde_json::from_str(&json)
        .map_err(|e| PacketProcessError::DeserializeError(Some(e.into())))?;
    let message = match LiveMessage::try_from(raw_live_message) {
        Ok(x) => x,
        Err(RawMessageDeserializeError::DeserializeError) => {
            log::debug!(target: "client", "Failed to deserialize raw message into specific message");
            return Err(PacketProcessError::DeserializeError(
                Some(RawMessageDeserializeError::DeserializeError.into())
            ));
        },
        Err(RawMessageDeserializeError::NotSupported(cmd)) => {
            log::debug!(target: "client", "Message type {:#} is not supported, ignored", cmd);
            return Ok(());
        }
    };
    log::info!(target: "client", "Message: {:#?}", message);   

    Ok(())
}