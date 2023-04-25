use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, SocketAddr};
use std::sync::Arc;
use io_arc::IoArc;
use log::{debug, warn};
use mio::net::{TcpStream, UdpSocket};
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, ActorResult, Handler};

pub struct NetWorker {
    streams: HashMap<usize, (IoArc<TcpStream>, SocketAddr)>,
    sockets: HashMap<usize, IoArc<UdpSocket>>,
}

impl NetWorker {
    pub fn new() -> Self {
        return Self {
            streams: HashMap::new(),
            sockets: HashMap::new(),
        };
    }
}
impl Actor for NetWorker {
    fn on_system_stop(&mut self, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        //we intentionally ignore if the actor system is stopped
        //we only react if the actor is explicitly stopped by the manager, because there might still be open connections that we don't want to drop
        Ok(ActorResult::Ok)
    }
}

pub struct NetWorkerFactory {}
impl ActorFactory<NetWorker> for NetWorkerFactory {
    fn new_actor(&mut self, _context: ActorContext<NetWorker>) -> Result<NetWorker, Box<dyn Error>> {
        return Ok(NetWorker::new());
    }
}

impl NetWorkerFactory {
    pub fn new() -> Self {
        return Self {

        };
    }
}

pub struct AddTcpConnection {
    pub stream_id: usize,
    pub stream: IoArc<TcpStream>,
    pub address: SocketAddr,
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

impl ActorMessage for AddTcpConnection {
    fn get_id(&self) -> usize {
        return self.stream_id;
    }
}

impl RemoveTcpConnection {
    pub fn new(stream_id: usize) -> Self {
        return Self {
            stream_id,
        };
    }
}

impl ActorMessage for RemoveTcpConnection {
    fn get_id(&self) -> usize {
        return self.stream_id;
    }
}

pub struct ReceiveTcpMessage {
    pub stream_id: usize,
    pub request: Vec<String>,
}

impl ReceiveTcpMessage {
    pub fn new(stream_id: usize, request: Vec<String>) -> Self {
        return Self {
            stream_id,
            request,
        };
    }
}

impl ActorMessage for ReceiveTcpMessage {
    fn get_id(&self) -> usize {
        return self.stream_id;
    }
}

pub struct AddUdpSocket {
    socket_id: usize,
    socket: IoArc<UdpSocket>,
}
impl AddUdpSocket {
    pub fn new(socket_id: usize, socket: IoArc<UdpSocket>) -> Self {
        return Self {
            socket_id,
            socket,
        };
    }
}

impl ActorMessage for AddUdpSocket {
    fn get_id(&self) -> usize {
        return 1;
    }
}

pub struct ReceiveUdpMessage {
    socket_id: usize,
    source: SocketAddr,
    request: String,
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

impl ActorMessage for ReceiveUdpMessage {
    fn get_id(&self) -> usize {
        return 1;
    }
}

impl Handler<ReceiveTcpMessage> for NetWorker {
    fn handle(&mut self, msg: ReceiveTcpMessage, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let stream = self.streams.get_mut(&msg.stream_id);
        if stream.is_none() {
            // temporary implementation for our instant http response, later on we won't have to care here if the stream is active, we'll just forward the message
            debug!("Stream ID no longer exists, can't reply to request");
            return Ok(ActorResult::Ok);
        }
        let (stream, _) = stream.unwrap();
        stream.write_all("HTTP/1.1 200 OK\nContent-Type: text/html\nConnection: keep-alive\nContent-Length: 12\r\n\r\nHELLO-WORLD!".as_bytes()).unwrap();

        // temporary implementation for our instant http response
        // drops the connection if keep-alive has not been specified
        let mut shutdown_connection = true;
        for k in msg.request {
            if k == "Connection: Keep-Alive" {
                shutdown_connection = false;
                break;
            }
        }
        if shutdown_connection {
            let _ = stream.as_ref().shutdown(Shutdown::Both);
        }

        return Ok(ActorResult::Ok);
    }
}

impl Handler<AddTcpConnection> for NetWorker {
    fn handle(&mut self, msg: AddTcpConnection, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let key_already_exists = self.streams.remove(&msg.stream_id);
        if key_already_exists.is_some() {
            warn!("Stream ID already exists, dropping old  one in favor of the new connection.");
            let (stream, _) = key_already_exists.unwrap();
            let _ = stream.as_ref().shutdown(Shutdown::Both);
        }

        let _ = self.streams.insert(msg.stream_id, (msg.stream, msg.address));
        return Ok(ActorResult::Ok);
    }
}

impl Handler<RemoveTcpConnection> for NetWorker {
    fn handle(&mut self, msg: RemoveTcpConnection, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let _ = self.streams.remove(&msg.stream_id);
        return Ok(ActorResult::Ok);
    }
}

impl Handler<ReceiveUdpMessage> for NetWorker {
    fn handle(&mut self, msg: ReceiveUdpMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let socket = self.sockets.get_mut(&msg.socket_id);
        if socket.is_none() {
            // temporary implementation for our instant http response, later on we won't have to care here if the stream is active, we'll just forward the message
            debug!("Socket ID no longer exists, can't reply to request");
            return Ok(ActorResult::Ok);
        }
        let socket = socket.unwrap();
        let _ = socket.as_ref().send_to(msg.request.as_bytes(), msg.source);

        return Ok(ActorResult::Ok);
    }
}

impl Handler<AddUdpSocket> for NetWorker {
    fn handle(&mut self, msg: AddUdpSocket, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let key_already_exists = self.sockets.remove(&msg.socket_id);
        if key_already_exists.is_some() {
            warn!("Socket ID already exists, dropping old  one in favor of the new.");
            let socket = key_already_exists.unwrap();
        }

        let _ = self.sockets.insert(msg.socket_id, msg.socket);
        return Ok(ActorResult::Ok);
    }
}