use std::io::Read;

use crate::{deserialize_packet, message::RawLiveMessage, PacketHeader, PacketType, Protocol};

pub enum DepackedMessage {
    CertificateResp,
    HeartbeatResp(u64),
    LiveMessages(Vec<RawLiveMessage>)
}

#[derive(Debug)]
pub enum PacketDepackError {
    WrongType,
    DecompressError,
    PacketDeserializeError,
    DeserializeError(Option<Box<dyn std::error::Error>>)
}

impl std::fmt::Display for PacketDepackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PacketProcessError {{ type: ")?;
        match self {
            Self::WrongType => f.write_str("WrongType")?,
            Self::DecompressError => f.write_str("DecompressionError")?,
            Self::PacketDeserializeError => f.write_str("PacketDeserializeError")?,
            Self::DeserializeError(e) => f.write_str(
                format!("DeserializeError, innerError: {:?}", e).as_str()
            )?
        };
        write!(f, "}}")
    }
}

impl std::error::Error for PacketDepackError {
    fn description(&self) -> &str {
        match self {
            Self::WrongType => "Packet type does not match",
            Self::DecompressError => "Failed to decompress data",
            Self::PacketDeserializeError => "Failed to deserialize packet header and body",
            Self::DeserializeError(_) => "Failed to deserialize packet body",
        }
    }
}

fn resolve_command_packet(header: PacketHeader, body: &[u8]) -> Result<RawLiveMessage, PacketDepackError> {
    if header.packet_type != PacketType::Command as u32 {
        return Err(PacketDepackError::WrongType);
    }
    let json = String::from_utf8(body.to_vec())
        .map_err(|e| PacketDepackError::DeserializeError(Some(e.into())))?;
    log::trace!(target: "client", "Processing JSON string: {:#?}", json);
    let raw_live_message: RawLiveMessage = serde_json::from_str(&json)
        .map_err(|e| PacketDepackError::DeserializeError(Some(e.into())))?;
    Ok(raw_live_message)
}

pub fn depack_packets(header: PacketHeader, body: &[u8]) -> Result<DepackedMessage, PacketDepackError> {
    if header.protocol == Protocol::CommandBrotli as u16 {
        let mut data: Vec<u8> = vec![];
        brotli::Decompressor::new(body, 4096)
            .read_to_end(&mut data)
            .map_err(|_| PacketDepackError::DecompressError)?;
        let total_length: usize = data.len();
        let mut read_len: usize = 0;
        let mut live_messages = vec![];
        while read_len < total_length {
            let (header, body) = match deserialize_packet(&data) {
                Ok(x) => x,
                Err(_) => { return Err(PacketDepackError::PacketDeserializeError); }
            };
            read_len += body.len() + 16;
            if header.protocol != Protocol::Command as u16 {
                log::debug!(target: "client", "Ignored non-command packet in inner packets");
                continue;
            }
            live_messages.push(resolve_command_packet(header, body)?);
        };
        Ok(DepackedMessage::LiveMessages(live_messages))
    } else if header.protocol == Protocol::CommandZlib as u16 {
        todo!()
    } else {
        if header.packet_type == PacketType::CertificateResp as u32 {
            Ok(DepackedMessage::CertificateResp)
        } else if header.packet_type == PacketType::HeartbeatResp as u32 {
            if body.len() <= 4 {
                Err(PacketDepackError::DeserializeError(None))
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