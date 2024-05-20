use derive_more::Display;

use bincode::Options;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[repr(u16)]
pub enum Protocol {
    Command         = 0,
    Special         = 1,
    CommandZlib     = 2,
    CommandBrotli   = 3
}

#[allow(dead_code)]
#[repr(u32)]
pub enum PacketType {
    Heartbeat       = 2,
    HeartbeatResp   = 3,
    Command         = 5,
    Certificate     = 7,
    CertificateResp = 8,
}

#[allow(dead_code)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificatePacketBody {
    pub uid: u64,
    pub roomid: u64,
    pub key: String,
    pub protover: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRespBody {
    pub code: i32
}

pub fn create_packet(
    protocol: Protocol, 
    packet_type: PacketType, 
    body: &[u8]
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let config = bincode::options()
        .with_fixint_encoding()
        .with_big_endian();
    let header = PacketHeader {
        total_size: (body.len() + 16) as u32,
        head_size: 16,
        protocol: protocol as u16,
        packet_type: packet_type as u32,
        sequence: 1
    };
    let header_data = config.serialize(&header)?;
    Ok([header_data, body.to_vec()].concat())
}

pub fn deserialize_packet(data: &[u8]) -> Result<(PacketHeader, &[u8]), Box<dyn std::error::Error>> {
    if data.len() < 16 {
        return Err(DeserializeFailedError {}.into());
    }
    let config = bincode::options()
        .with_fixint_encoding()
        .with_big_endian();
    let header: PacketHeader = config.deserialize(&data[..16])?;
    let total_size = header.total_size as usize;
    if data.len() < total_size {
        return Err(DeserializeFailedError {}.into());
    }
    Ok((header, &data[16..total_size as usize]))
}

#[derive(Debug, Display)]
pub struct DeserializeFailedError;

impl std::error::Error for DeserializeFailedError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
    fn description(&self) -> &str {
        "Failed to deserialize packet"
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn create_certificate_packet(
    uid: u64, 
    room_id: u64, 
    token: &str
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cert_body = CertificatePacketBody {
        uid,
        roomid: room_id,
        key: token.to_string(),
        protover: Protover::Brotli as u8
    };
    let cert_body = serde_json::ser::to_string(&cert_body)?;
    create_packet(
        Protocol::Special, 
        PacketType::Certificate, 
        cert_body.as_bytes()
    )
}

pub fn create_heartbeat_packet() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let body = b"[Object object]";
    create_packet(Protocol::Special, PacketType::Heartbeat, body)
}