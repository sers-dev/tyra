use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use io_arc::IoArc;
use mio::net::{TcpStream, UdpSocket};
use crate::prelude::ActorMessage;

pub struct AddTcpConnection {
    pub stream_id: usize,
    pub stream: IoArc<TcpStream>,
    pub address: SocketAddr,
}

impl Hash for AddTcpConnection
{
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

pub struct RemoveTcpConnection {
    pub stream_id: usize,
}

impl Hash for RemoveTcpConnection
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
    }
}

impl ActorMessage for AddTcpConnection {}

impl RemoveTcpConnection {
    pub fn new(stream_id: usize) -> Self {
        return Self {
            stream_id,
        };
    }
}

impl ActorMessage for RemoveTcpConnection {}

pub struct ReceiveTcpMessage {
    pub stream_id: usize,
    pub request: Vec<String>,
}

impl Hash for ReceiveTcpMessage
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream_id.hash(state);
    }
}

impl ReceiveTcpMessage {
    pub fn new(stream_id: usize, request: Vec<String>) -> Self {
        return Self {
            stream_id,
            request,
        };
    }
}

impl ActorMessage for ReceiveTcpMessage {}

pub struct AddUdpSocket {
    pub socket_id: usize,
    pub socket: IoArc<UdpSocket>,
}

impl Hash for AddUdpSocket
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.socket_id.hash(state);
    }
}

impl AddUdpSocket {
    pub fn new(socket_id: usize, socket: IoArc<UdpSocket>) -> Self {
        return Self {
            socket_id,
            socket,
        };
    }
}

impl ActorMessage for AddUdpSocket {}

pub struct ReceiveUdpMessage {
    pub socket_id: usize,
    pub source: SocketAddr,
    pub request: String,
}

impl Hash for ReceiveUdpMessage
{
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