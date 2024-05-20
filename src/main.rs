use bincode::{Options};
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use reqwest::Body;
use simple_logger::SimpleLogger;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::{env, thread::sleep, time::Duration};
use tokio;

mod packet;

use packet::{http::*, ws::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_level(log::LevelFilter::Info).env().with_timestamp_format(
        time::macros::format_description!("[hour]:[minute]:[second]")
    ).init().unwrap();
    // Get arguments
    let args: Vec<String> = env::args().collect();

    let room_id = args
        .iter()
        .enumerate()
        .find_map(|(index, label)| {
            if label.starts_with("--id=") {
                let id_str = label
                    .split('=')
                    .next()
                    .expect("Room ID argument is required!");
                let id = id_str
                    .parse::<usize>()
                    .expect(&format!("Invalid room ID \"{}\"!", id_str));
                Some(id)
            } else if label.starts_with("--id") {
                let id_str = args.get(index + 1).expect("Room ID argument is required!");
                let id = id_str
                    .parse::<usize>()
                    .expect(&format!("Invalid room ID \"{}\"!", id_str));
                Some(id)
            } else {
                None
            }
        })
        .expect("Room ID is not provided!");

    // Start calling APIs

    let room_data: RoomInitData = reqwest::get(format!(
        "https://api.live.bilibili.com/room/v1/Room/room_init?id={}",
        room_id
    )).await?
        .json::<HttpAPIResponse<RoomInitData>>().await?
        .response_data()
        .expect("Invalid room_init response data.");
    
    let room_id = room_data.room_id;

    log::info!(
        target: "init",
        "Requested real room ID: {}", room_id.to_string().bright_green()
    );

    let danmaku_info_data: DanmakuInfoData = reqwest::get(format!(
        "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}",
        room_id
    )).await?
        .json::<HttpAPIResponse<DanmakuInfoData>>().await?
        .response_data()
        .expect("Invalid danmaku_info response data.");
    log::info!(
        target: "init",
        "Requested token and WebSocket servers. {} servers available.",
        danmaku_info_data.host_list.len().to_string().bright_green()
    );

    // Get token and host uri
    let token = danmaku_info_data.token;
    let host = danmaku_info_data.host_list.get(0).expect("No available server in the list!").clone();
    let host_url = format!("wss://{}:{}/sub", host.host, host.wss_port);
    log::info!(
        target: "init",
        "Initializing connection to {} ...",
        host_url.bright_green()
    );
    let host_url = url::Url::parse(&host_url).expect("Failed to parse URL");
    
    loop {
        if let Err(e) = start_listening(room_id, &token, &host_url).await {
            log::warn!(target: "init", "Connection closed! \n {}", e.to_string());
            log::warn!(target: "init", "Trying to reconnect after 5 seconds.");
            sleep(Duration::from_secs(5));
        }
    }
}

async fn start_listening(
    room_id: u64,
    token: &String,
    host_url: &url::Url
) -> Result<(), Box<dyn std::error::Error>> {

    let (mut stream, _) = connect_async(host_url).await?;
    log::info!(
        target: "init",
        "Successfully connected to server"
    );
    // Create certificate body
    let cert_body = CertificatePacketBody {
        uid: 0,
        roomid: room_id,
        key: token.to_string(),
        protover: Protover::Brotli as u8
    };
    let cert_body = serde_json::ser::to_string(&cert_body)?;
    let packet_data = create_packet(
        Protocol::Special, 
        PacketType::Certificate, 
        cert_body.as_bytes()
    )?;
    stream.send(Message::binary(packet_data)).await?;
    
    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                log::warn!("Failed to receive message: {}", e);
                continue;
            }
        };
        let (header, body) = match deserialize_packet(msg.into_data().as_slice()) {
            Ok(x) => x,
            Err(e) => {
                log::warn!("Failed to deserialize message: {}", e);
                continue;
            }
        };

        log::debug!("Received packet: \n{:#?}\nBody: {}", header, hex::encode(body.clone()));
        process_packet(header, body.as_slice());
    }

    loop {
        
    }
}

fn process_packet(header: PacketHeader, body: &[u8]) {
    
}