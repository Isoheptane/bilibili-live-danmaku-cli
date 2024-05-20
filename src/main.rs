use chrono::{TimeDelta, Utc};
use colored::Colorize;
use futures_util::{FutureExt, SinkExt, StreamExt};
use simple_logger::SimpleLogger;
use tokio_tungstenite::{connect_async, tungstenite::Message};
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
    let (mut stream, _) = connect_async(host_url).await?;
    log::info!(
        target: "client",
        "Successfully connected to server"
    );
    let mut last_heartbeat = Utc::now();
    // Send certificate
    stream.send(Message::binary(create_certificate_packet(uid, room_id, token)?)).await?;
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
                match stream.send(Message::binary(packet)).await {
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
        if let Some(Some(msg)) = stream.next().now_or_never() {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    log::warn!(
                        target: "client", 
                        "Failed to receive message: {}", 
                        e
                    );
                    continue;
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

fn process_packet(header: PacketHeader, body: &[u8]) {
    if header.protocol == Protocol::CommandBrotli as u16 {
        let mut data: Vec<u8> = vec![];
        if let Err(e) = brotli::Decompressor::new(body, 4096).read_to_end(&mut data) {
            log::debug!(target: "client", "Failed to decompress data: {}", e);
        }
        let total_length: usize = data.len();
        let mut read_len: usize = 0;
        while read_len < total_length {
            let (header, body) = match deserialize_packet(&data[read_len..]) {
                Ok(x) => x,
                Err(e) => {
                    log::debug!(
                        target: "client", 
                        "Failed to deserialize inner message: {}", 
                        e
                    );
                    break;
                }
            };
            read_len += body.len() + 16;
            process_packet(header, body);
            log::debug!(
                target: "client", 
                "Processed inner message block. Read length {}/{} bytes.",
                read_len,
                total_length
            );
        }
        return;
    } else if header.protocol == Protocol::CommandZlib as u16 {
        todo!();
        return;
    }
    // Raw packet process
    // Heartbeat 
    if header.packet_type != PacketType::Command as u32 {
        //  TODO
        return;
    }
    // Add ending zero to make sure String::from_utf8 will work
    let mut bytes = body.to_vec();
    bytes.push(0);
    let json = match String::from_utf8(bytes) {
        Ok(json) => json,
        Err(_) => { return; }
    };
}