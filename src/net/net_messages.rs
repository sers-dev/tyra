use crate::prelude::ActorMessage;
use io_arc::IoArc;
use mio::net::{TcpStream, UdpSocket};
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;

#[derive(Serialize)]
pub struct AddTcpConnection {
    pub stream_id: usize,
    #[serde(skip)]
    pub stream: IoArc<TcpStream>,
    pub address: SocketAddr,
}

impl Hash for AddTcpConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
    }
}

impl AddTcpConnection {
    pub fn new(stream_id: usize, stream: IoArc<TcpStream>, address: SocketAddr) -> Self {
        return Self {
            stream_id,
            stream,
            address,
        };
    }
}

#[derive(Serialize)]
pub struct RemoveTcpConnection {
    pub stream_id: usize,
}

impl Hash for RemoveTcpConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
    }
}

impl ActorMessage for AddTcpConnection {}

impl RemoveTcpConnection {
    pub fn new(stream_id: usize) -> Self {
        return Self { stream_id };
    }
}

impl ActorMessage for RemoveTcpConnection {}

#[derive(Serialize)]
pub struct ReceiveTcpMessage {
    pub stream_id: usize,
    pub request: Vec<String>,
}

impl Hash for ReceiveTcpMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
    }
}

impl ReceiveTcpMessage {
    pub fn new(stream_id: usize, request: Vec<String>) -> Self {
        return Self { stream_id, request };
    }
}

impl ActorMessage for ReceiveTcpMessage {}

#[derive(Serialize)]
pub struct AddUdpSocket {
    pub socket_id: usize,
    #[serde(skip)]
    pub socket: IoArc<UdpSocket>,
}

impl Hash for AddUdpSocket {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.socket_id.hash(state);
    }
}

impl AddUdpSocket {
    pub fn new(socket_id: usize, socket: IoArc<UdpSocket>) -> Self {
        return Self { socket_id, socket };
    }
}

impl ActorMessage for AddUdpSocket {}

#[derive(Serialize)]
pub struct ReceiveUdpMessage {
    pub socket_id: usize,
    #[serde(skip)]
    pub source: SocketAddr,
    pub request: String,
}

impl Hash for ReceiveUdpMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.socket_id.hash(state);
    }
}

impl ReceiveUdpMessage {
    pub fn new(socket_id: usize, source: SocketAddr, request: String) -> Self {
        return Self {
            socket_id,
            source,
            request,
        };
    }
}

impl ActorMessage for ReceiveUdpMessage {}
