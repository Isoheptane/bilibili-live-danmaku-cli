use std::io::Read;
use derive_more::Display;

use crate::{message::RawLiveMessage, Packet, PacketConvertError, PacketHeader, PacketType, Protocol};

pub enum DepackedMessage {
    CertificateResp,
    HeartbeatResp(u64),
    LiveMessages(Vec<RawLiveMessage>)
}

#[derive(Debug, Display)]
pub enum PacketDepackError {
    WrongType,
    DecompressError,
    PacketConvertError(PacketConvertError),
    BodyDeserializeError,
}

impl std::error::Error for PacketDepackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::WrongType => None,
            Self::DecompressError => None,
            Self::PacketConvertError(e) => Some(e),
            Self::BodyDeserializeError => None,
        }
    }
}

fn resolve_command_packet(header: PacketHeader, body: &[u8]) -> Result<RawLiveMessage, PacketDepackError> {
    if header.packet_type != PacketType::Command as u32 {
        return Err(PacketDepackError::WrongType);
    }
    // log::debug!("Resolving command packet: {:#?}\n{:?}", header, body);
    let json = String::from_utf8(body.to_vec())
        .map_err(|_| PacketDepackError::BodyDeserializeError)?;
    log::trace!(target: "client", "Processing JSON string: {:#?}", json);
    let raw_live_message: RawLiveMessage = serde_json::from_str(&json)
        .map_err(|_| PacketDepackError::BodyDeserializeError)?;
    Ok(raw_live_message)
}

pub fn depack_packets(header: PacketHeader, body: &[u8]) -> Result<DepackedMessage, PacketDepackError> {
    if header.protocol == Protocol::CommandBrotli as u16 {
        // Brotli compressed live message
        let mut data: Vec<u8> = vec![];
        brotli::Decompressor::new(body, 4096)
            .read_to_end(&mut data)
            .map_err(|_| PacketDepackError::DecompressError)?;
        let total_length: usize = data.len();
        let mut read_len: usize = 0; 
        let mut live_messages = vec![];
        while read_len < total_length {
            // Read depacked message from read_len
            let packet = match Packet::from_binary(&data[read_len..]) {
                Ok(x) => x,
                Err(e) => { return Err(PacketDepackError::PacketConvertError(e)); }
            };
            // Read length
            read_len += packet.header.total_size as usize;
            if packet.header.protocol != Protocol::Command as u16 {
                log::debug!(target: "client", "Ignored non-command packet in inner packets");
                continue;
            }
            live_messages.push(resolve_command_packet(packet.header, &packet.body)?);
        };
        Ok(DepackedMessage::LiveMessages(live_messages))
    } else if header.protocol == Protocol::CommandZlib as u16 {
        // Zlib compressed live message
        todo!()
    } else {
        if header.packet_type == PacketType::CertificateResp as u32 {
            Ok(DepackedMessage::CertificateResp)
        } else if header.packet_type == PacketType::HeartbeatResp as u32 {
            if body.len() < 4 {
                Err(PacketDepackError::BodyDeserializeError)
            } else {
                let count = 
                    ((body[3] as u64) << 0) + 
                    ((body[2] as u64) << 8) +
                    ((body[1] as u64) << 16) +
                    ((body[0] as u64) << 24);
                Ok(DepackedMessage::HeartbeatResp(count))
            }
        } else if header.packet_type == PacketType::Command as u32 {
            resolve_command_packet(header, body).map(|command| DepackedMessage::LiveMessages(vec![command]))
        } else {
            Err(PacketDepackError::WrongType)
        }
    }
}