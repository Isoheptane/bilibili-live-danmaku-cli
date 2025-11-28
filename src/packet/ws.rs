use derive_more::Display;

use bincode::Options;
use serde::{Deserialize, Serialize};

#[repr(u16)]
pub enum Protocol {
    Command         = 0,
    Special         = 1,
    CommandZlib     = 2,
    CommandBrotli   = 3
}

impl TryFrom<u16> for Protocol {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Protocol::Command),
            1 => Ok(Protocol::Special),
            2 => Ok(Protocol::CommandZlib),
            3 => Ok(Protocol::CommandBrotli),
            _ => Err(())
        }
    }
}

#[repr(u32)]
pub enum PacketType {
    Heartbeat       = 2,
    HeartbeatResp   = 3,
    Command         = 5,
    Certificate     = 7,
    CertificateResp = 8,
}

impl TryFrom<u32> for PacketType {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(PacketType::Heartbeat),
            3 => Ok(PacketType::HeartbeatResp),
            5 => Ok(PacketType::Command),
            7 => Ok(PacketType::Certificate),
            8 => Ok(PacketType::CertificateResp),
            _ => Err(())
        }
    }
}

#[allow(unused)]
#[repr(u8)]
pub enum Protover {
    Normal = 1,
    Zlib   = 2,
    Brotli = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub total_size: u32,
    pub head_size: u16,
    pub protocol: u16,
    pub packet_type: u32,
    pub sequence: u32,
}

// Note: Certificate packet body is in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificatePacketBody {
    pub uid: u64,
    pub roomid: u64,
    pub key: String,
    pub protover: u8,
}

// It's fine to ignore the certificate response body so it is marked allow(unused)
#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRespBody {
    pub code: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub header: PacketHeader,
    pub body: Vec<u8>
}

impl Packet {
    pub fn create(protocol: Protocol, packet_type: PacketType, body: Vec<u8>) -> Self {
        let header = PacketHeader {
            total_size: (body.len() + 16) as u32,
            head_size: 16,
            protocol: protocol as u16,
            packet_type: packet_type as u32,
            sequence: 1
        };
        Packet { header, body }
    }

    pub fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        let config = bincode::options()
        .with_fixint_encoding()
        .with_big_endian();
        let header_binary = config.serialize(&self.header)?;
        Ok([header_binary, self.body.clone()].concat())
    }

    pub fn from_binary(data: &[u8]) -> Result<Packet, PacketConvertError> {
        if data.len() < 16 {
            return Err(PacketConvertError::PacketLengthError);
        }
        let config = bincode::options()
        .with_fixint_encoding()
        .with_big_endian();
        let header: PacketHeader = config.deserialize(&data[..16]).map_err(|e| PacketConvertError::BinCodeError(e))?;
        let total_size = header.total_size as usize;
        if data.len() < total_size {
            return Err(PacketConvertError::PacketLengthError);
        }
        let body = data[16..total_size as usize].to_vec();
        Ok(Packet{ header, body })
    }

    pub fn new_certificate_packet(uid: u64, room_id: u64, token: &str) -> Result<Packet, PacketConvertError> {
        let cert_body = CertificatePacketBody {
            uid,
            roomid: room_id,
            key: token.to_string(),
            // Protover is set to Brotli, not tested on Zlib or Normal
            protover: Protover::Brotli as u8
        };
        let cert_body = serde_json::ser::to_string(&cert_body).map_err(|_| PacketConvertError::BodySerializeError)?;
        Ok(Packet::create(Protocol::Special, PacketType::Certificate, cert_body.as_bytes().to_vec()))
    }
}

#[derive(Debug, Display)]
pub enum PacketConvertError {
    PacketLengthError,
    BodySerializeError,
    BinCodeError(bincode::Error),
}

impl std::error::Error for PacketConvertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::PacketLengthError => None,
            Self::BodySerializeError => None,
            Self::BinCodeError(e) => Some(e)
        }
    }
}

pub fn heartbeat_packet_binary() -> &'static [u8] {
    &[
        0, 0, 0, 31,    // Total length 
        0, 16,          // Header size
        0, 1,           // Protocol
        0, 0, 0, 2,     // Packet type
        0, 0, 0, 1,     // Sequence
        91, 79, 98, 106, 101, 99, 116, 32, 111, 98, 106, 101, 99, 116, 93   // Body
    ]
}