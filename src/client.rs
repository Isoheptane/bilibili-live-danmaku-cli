use std::error;
use std::net::TcpStream;

use derive_more::Display;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::depack::{depack_packets, DepackedMessage};
use crate::session_data::SessionData;
use crate::Packet;

#[allow(unused)]
pub struct LiveClient {
    client: WebSocket<MaybeTlsStream<TcpStream>>,
    connected: bool,
    session: SessionData
}

#[derive(Debug, Display)]
pub enum ClientError {
    TungsteniteError(tungstenite::Error),
    IOError(std::io::Error),
    ConnectionClosed,
    PacketProcessError
}

impl error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::TungsteniteError(e) => Some(e),
            Self::IOError(e) => Some(e),
            Self::ConnectionClosed => None,
            Self::PacketProcessError => None
        }
    }
}

impl LiveClient {
    pub fn new(host_url: &str, session: SessionData) -> Result<Self, ClientError> {
        
        let (mut client, _) = tungstenite::connect(host_url)
            .map_err(|e| ClientError::TungsteniteError(e))?;

        match client.get_mut() {
            MaybeTlsStream::Plain(stream) => {
                stream.set_nonblocking(true)
            },
            MaybeTlsStream::NativeTls(stream) => {
                stream.get_mut().set_nonblocking(true)
            },
            _ => unimplemented!()
        }
        .map_err(|e| ClientError::IOError(e))?;

        let certificate = Packet::new_certificate_packet(session.uid, session.room_id, &session.token)
            .map_err(|_| ClientError::PacketProcessError)?
            .to_binary()
            .map_err(|_| ClientError::PacketProcessError)?;

        client.send(Message::binary(certificate))
        .map_err(|e|ClientError::TungsteniteError(e))?;
        log::debug!("Certificate packet sent");

        Ok(LiveClient{ client, session, connected: true })
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), ClientError> {
        if !self.connected {
            return Err(ClientError::ConnectionClosed)
        }
        log::debug!("Message send invoked");
        self.client.send(message).map_err(|e| ClientError::TungsteniteError(e))
    }

    pub fn recv_messages(&mut self) -> Result<Vec<DepackedMessage>, ClientError> {
        let mut messages: Vec<DepackedMessage> = vec![];
        // Read all packets
        loop {
            let msg = match self.client.read() {
                Ok(x) => x,
                Err(tungstenite::Error::Io(e)) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        return Ok(messages);
                    }
                    return Err(ClientError::TungsteniteError(tungstenite::Error::Io(e)));
                },
                Err(e) => {
                    return Err(ClientError::TungsteniteError(e));
                }
            };
            if msg.is_close() {
                self.connected = false;
                return Ok(messages);
            }
            let data = msg.into_data();
            let packet = match Packet::from_binary(data.as_slice()) {
                Ok(x) => x,
                Err(_) => { continue; }
            };
            log::debug!(
                target: "client", 
                "Received packet: {:?}",
                packet.header
            );
            let message = match depack_packets(packet.header, &packet.body) {
                Ok(message) => message, 
                Err(e) => {
                    log::debug!(target: "client", "Failed to depack packets: {}", e);
                    continue;
                }
            };
            messages.push(message);
        };
    }
}