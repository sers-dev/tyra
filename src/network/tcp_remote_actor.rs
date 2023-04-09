use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::thread::{sleep, Thread};
use std::time::Duration;
use threadpool::ThreadPool;
use crate::actor::actor_address::ActorAddress;
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorInitMessage, ActorMessage, ActorResult, Handler, SerializedMessage};

struct NetworkMessage {
    address: ActorAddress,
    content: SerializedMessage,
    id: u32,
    stream: TcpStream
}

impl ActorMessage for NetworkMessage {}

pub struct TcpRemoteActor {
    listener: Arc<TcpListener>,
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
}
impl TcpRemoteActor {
    pub fn new() -> Self {
        println!("SERS!");
        let listener = Arc::new(TcpListener::bind("127.0.0.1:2022").unwrap());
        println!("SERS!");


        Self {
            listener,
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl Actor for TcpRemoteActor {
    fn pre_stop(&mut self, _context: &ActorContext<Self>) {
        println!("PRE-STOP!");
        sleep(Duration::from_secs(3));
        self.is_stopping.store(true, Ordering::Relaxed);
    }
    fn post_stop(&mut self, _context: &ActorContext<Self>) {
        println!("POST-STOP!");
        self.is_stopped.store(true, Ordering::Relaxed);
    }
    fn on_system_stop(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        println!("SYSTEM-STOP");
        Ok(ActorResult::Stop)
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
        context: ActorContext<TcpRemoteActor>,
    ) -> Result<TcpRemoteActor, Box<dyn Error>> {
        context.actor_ref.send(ActorInitMessage::new()).unwrap();
        Ok(TcpRemoteActor::new())
    }
}

// implement our message for the `Actor`
impl Handler<ActorInitMessage> for TcpRemoteActor {
    fn handle(
        &mut self,
        _msg: ActorInitMessage,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        println!("AAAAA");

        let listener = self.listener.clone();
        let is_stopped = self.is_stopped.clone();
        let is_stopping = self.is_stopping.clone();
        let context = _context.clone();
        thread::spawn(move || {

            let mut count = AtomicUsize::new(1);


            let pool = ThreadPool::new(100);
            for stream in listener.incoming() {
                //println!("SERS!");
                if is_stopped.load(Ordering::Relaxed) {
                    println!("EXIT");
                    return;
                }
                if is_stopping.load(Ordering::Relaxed) {
                    println!("NO MORE NEW CONNECTIONS");
                    continue;
                }
                let counter = count.fetch_add(1, Ordering::Relaxed);
                println!("NEW CONNECTION!: {:?}", counter);

                let context = context.clone();
                let mut stream = stream.unwrap();
                pool.execute(move || {
                    //let context = &context;

                    let mut buf_reader = BufReader::new(&mut stream);
                    let mut http_req = String::new();
                    //let http_request = buf_reader.read_line(&mut http_req);
                    let http_request: Vec<_> = buf_reader
                        .lines()
                        .map(|result| result.unwrap())
                        .take_while(|line| !line.is_empty())
                        .collect();

                    //println!("Request: {:#?}", http_request);
                    context.actor_ref.send(NetworkMessage{
                        id: 1,
                        stream: stream,
                        address: context.actor_ref.get_address().clone(),
                        content: SerializedMessage::new(Vec::new()),
                    }).unwrap();
                })
            }
        });


        Ok(ActorResult::Ok)
    }
}

impl Handler<NetworkMessage> for TcpRemoteActor {
    fn handle(&mut self, mut msg: NetworkMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        //println!("HELLO WORLD: {:?}", msg.address.actor);
        //println!("ASDF: {:?}", msg.content);
        let context = context.clone();
        thread::spawn(move || {

            sleep(Duration::from_secs(3));

            let response = "HTTP/1.1 200 OK\r\n\r\n<h1>SERS</h1>";

            msg.stream.write_all(response.as_bytes()).unwrap();

            context.system.stop(Duration::from_secs(30));


            //context.actor_ref.stop().unwrap();
        });
        Ok(ActorResult::Ok)

    }
}