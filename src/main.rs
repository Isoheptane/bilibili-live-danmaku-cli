use colored::Colorize;
use simple_logger::SimpleLogger;
use std::env;
use tokio;

mod packet;

use packet::*;

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

    let token = danmaku_info_data.token;

    let host = danmaku_info_data.host_list.get(0).expect("No available server in the list!").clone();

    log::info!(
        target: "init",
        "Initializing connection to {} ...",
        format!("wss://{}:{}", host.host, host.wss_port).bright_green()
    );

    
    
    Ok(())
}
