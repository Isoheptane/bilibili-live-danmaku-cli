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
    InvalidProtocol,
    InvalidType,
    DecompressError,
    PacketConvertError(PacketConvertError),
    BodyDeserializeError,
}

impl std::error::Error for PacketDepackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidProtocol => None,
            Self::InvalidType => None,
            Self::DecompressError => None,
            Self::PacketConvertError(e) => Some(e),
            Self::BodyDeserializeError => None,
        }
    }
}

fn resolve_command_packet(header: PacketHeader, body: &[u8]) -> Result<RawLiveMessage, PacketDepackError> {
    if header.packet_type != PacketType::Command as u32 {
        return Err(PacketDepackError::InvalidType);
    }
    // log::debug!("Resolving command packet: {:#?}\n{:?}", header, body);
    let json = String::from_utf8(body.to_vec())
        .map_err(|_| PacketDepackError::BodyDeserializeError)?;
    log::debug!(target: "client", "Processing JSON string:\n{}", json);
    let raw_live_message: RawLiveMessage = serde_json::from_str(&json)
        .map_err(|_| PacketDepackError::BodyDeserializeError)?;
    Ok(raw_live_message)
}

pub fn depack_packets(header: PacketHeader, raw_body: &[u8]) -> Result<DepackedMessage, PacketDepackError> {
    // Decompressed body data
    let mut body: Vec<u8> = vec![];

    let protocol: Protocol = match header.protocol.try_into() {
        Ok(proto) => proto,
        Err(_) => return Err(PacketDepackError::InvalidProtocol)
    };

    match protocol {
        Protocol::CommandBrotli => {
            brotli::Decompressor::new(raw_body, 4096)
                .read_to_end(&mut body)
                .map_err(|_| PacketDepackError::DecompressError)?;
        }
        Protocol::CommandZlib => {
            unimplemented!("Zlib compression is not currently supported.");
        }
        Protocol::Command | Protocol::Special => {
            body.extend_from_slice(raw_body);
        }
    }

    match protocol {
        // Single command or special command
        Protocol::Special | Protocol::Command => {
            if header.packet_type == PacketType::CertificateResp as u32 {
            return Ok(DepackedMessage::CertificateResp)
            } else if header.packet_type == PacketType::HeartbeatResp as u32 {
                if body.len() < 4 {
                    return Err(PacketDepackError::BodyDeserializeError)
                } else {
                    let count = 
                        ((body[3] as u64) << 0) + 
                        ((body[2] as u64) << 8) +
                        ((body[1] as u64) << 16) +
                        ((body[0] as u64) << 24);
                    return Ok(DepackedMessage::HeartbeatResp(count))
                }
            } else if header.packet_type == PacketType::Command as u32 {
                return resolve_command_packet(header, &body).map(|command| DepackedMessage::LiveMessages(vec![command]))
            } else {
                return Err(PacketDepackError::InvalidType)
            }
        }
        // Compressed command lists
        Protocol::CommandBrotli | Protocol::CommandZlib => {
            let total_length: usize = body.len();
            let mut read_len: usize = 0; 
            let mut live_messages = vec![];
            while read_len < total_length {
                // Read depacked message from read_len
                let packet = match Packet::from_binary(&body[read_len..]) {
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
            return Ok(DepackedMessage::LiveMessages(live_messages))
        }
    }
}