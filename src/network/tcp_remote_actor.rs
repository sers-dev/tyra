use std::error::Error;
use std::io::{Write};
use std::net::TcpStream;
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, ActorResult, Handler, SerializedMessage};

pub struct NetworkMessage {
    pub content: SerializedMessage,
    pub id: usize,
    pub stream: TcpStream
}

impl ActorMessage for NetworkMessage {}

pub struct TcpRemoteActor {}

impl TcpRemoteActor {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl Actor for TcpRemoteActor {
    fn pre_stop(&mut self, _context: &ActorContext<Self>) {
        println!("STOPPING TCP REMOTE ACTOR");
    }
    fn on_system_stop(&mut self, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        //we intentionally ignore if the actor system is stopped
        //we only react if the actor is explicitly stopped by the manager, because there might still be open connections that we don't want to drop
        Ok(ActorResult::Ok)
    }
}

// define a factory that creates the `Actor` for us
pub struct TcpRemoteActorFactory {}
impl TcpRemoteActorFactory {
    pub fn new() -> Self {
        Self {}
    }
}
impl ActorFactory<TcpRemoteActor> for TcpRemoteActorFactory {
    fn new_actor(
        &mut self,
        _context: ActorContext<TcpRemoteActor>,
    ) -> Result<TcpRemoteActor, Box<dyn Error>> {
        Ok(TcpRemoteActor::new())
    }
}

impl Handler<NetworkMessage> for TcpRemoteActor {
    fn handle(&mut self, mut msg: NetworkMessage, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        println!("RECEIVED MSG!:");
        let response = "HTTP/1.1 200 OK\r\n\r\n<h1>SERS</h1>";

        msg.stream.write_all(response.as_bytes()).unwrap();

        Ok(ActorResult::Ok)

    }
}