use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, SocketAddr};
use io_arc::IoArc;
use log::{debug, warn};
use mio::net::{TcpStream, UdpSocket};
use crate::net::net_messages::{AddTcpConnection, AddUdpSocket, ReceiveTcpMessage, ReceiveUdpMessage, RemoveTcpConnection};
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorResult, Handler};

#[derive(Clone)]
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

#[derive(Clone)]
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
    fn handle(&mut self, msg: ReceiveUdpMessage, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
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
    fn handle(&mut self, msg: AddUdpSocket, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let key_already_exists = self.sockets.remove(&msg.socket_id);
        if key_already_exists.is_some() {
            warn!("Socket ID already exists, dropping old  one in favor of the new.");
        }

        let _ = self.sockets.insert(msg.socket_id, msg.socket);
        return Ok(ActorResult::Ok);
    }
}