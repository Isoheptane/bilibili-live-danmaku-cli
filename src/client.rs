use std::error;
use std::io::ErrorKind;
use std::net::TcpStream;

use derive_more::Display;

use native_tls::TlsStream;
use websocket::sync::Client;
use websocket::ws::dataframe::DataFrame;
use websocket::{Message, WebSocketError};

use crate::depack::{depack_packets, DepackedMessage};
use crate::session_data::SessionData;
use crate::Packet;

#[allow(unused)]
pub struct LiveClient {
    client: Client<TlsStream<TcpStream>>,
    connected: bool,
    session: SessionData
}

#[derive(Debug, Display)]
pub enum Error {
    URIParseError(websocket::url::ParseError),
    CreateClientError(std::io::Error),
    WebSocketError(websocket::WebSocketError),
    ConnectionClosed,
    PacketProcessError
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::URIParseError(e) => Some(e),
            Self::CreateClientError(e) => Some(e),
            Self::WebSocketError(e) => Some(e),
            Self::ConnectionClosed => None,
            Self::PacketProcessError => None
        }
    }
}

impl LiveClient {
    pub fn new(host_url: &str, session: SessionData) -> Result<Self, Error> {
        let mut client = websocket::ClientBuilder::new(host_url)
            .map_err(|e| Error::URIParseError(e))?
            .connect_secure(None)
            .map_err(|e| Error::WebSocketError(e))?;
        // Client should work in nonblocking mode
        client.set_nonblocking(true).map_err(|e| Error::CreateClientError(e))?;

        let certificate = Packet::new_certificate_packet(session.uid, session.room_id, &session.token).map_err(|_| Error::PacketProcessError)?;
        client.send_message(
            &Message::binary(certificate.to_binary().map_err(|_| Error::PacketProcessError)?)
        ).map_err(|_| Error::PacketProcessError)?;

        Ok(LiveClient{ client, session, connected: true })
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), Error> {
        if !self.connected {
            return Err(Error::ConnectionClosed)
        }
        self.client.send_message(&message).map_err(|e| Error::WebSocketError(e))
    }

    pub fn recv_messages(&mut self) -> Result<Vec<DepackedMessage>, Error> {
        let mut messages: Vec<DepackedMessage> = vec![];
        // Read all packets
        let error = 'poll: loop {
            let msg = match self.client.recv_message() {
                Ok(x) => x,
                Err(e) => break 'poll e
            };
            if msg.is_close() {
                self.connected = false;
                return Err(Error::ConnectionClosed)
            }
            let data = msg.take_payload();
            let packet = match Packet::from_binary(data.as_slice()) {
                Ok(x) => x,
                Err(_) => { continue; }
            };
            log::trace!(
                target: "client", 
                "Received packet: {:?}",
                packet.header
            );
            let message = match depack_packets(packet.header, &packet.body) {
                Ok(message) => message, 
                Err(e) => {
                    log::debug!(target: "client", "Failed to depack packets: {}", e);
                    continue 'poll;
                }
            };
            messages.push(message);
        };
        // Fetch out websocket errors
        match error {
            WebSocketError::IoError(io_error) => {
                // Return messages on blocking operations
                if io_error.kind() == ErrorKind::WouldBlock {
                    return Ok(messages)
                } else {
                    return Err(Error::WebSocketError(WebSocketError::IoError(io_error)))
                }
            },
            WebSocketError::NoDataAvailable => {
                // Server disconnect
                self.connected = false;
                return Err(Error::ConnectionClosed)
            },
            e => {
                return Err(Error::WebSocketError(e))
            }
        };
    }
}