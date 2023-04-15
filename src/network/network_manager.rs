use std::error::Error;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use threadpool::ThreadPool;
use crate::network::tcp_remote_actor::{NetworkMessage, TcpRemoteActor, TcpRemoteActorFactory};
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorInitMessage, ActorMessage, ActorResult, ActorWrapper, Handler, SerializedMessage};
use crate::router::{AddActorMessage, LeastMessageRouter, LeastMessageRouterFactory};


pub struct NetworkManager {
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    actors: Vec<ActorWrapper<TcpRemoteActor>>,
    router: ActorWrapper<LeastMessageRouter<TcpRemoteActor>>,
    remote_actor_count: usize,
    graceful_shutdown_time_in_seconds: u64
}

struct NetworkManagerInitMessage {
}
impl ActorMessage for NetworkManagerInitMessage {}

impl NetworkManager {
    pub fn new(context: ActorContext<Self>, graceful_shutdown_time_in_seconds: u64) -> Self {

        context.actor_ref.send(ActorInitMessage::new()).unwrap();
        let router = context.system
            .builder()
            .set_pool_name(&context.actor_ref.get_address().pool)
            .spawn(format!("{}-tcp-router", context.actor_ref.get_address().actor), LeastMessageRouterFactory::new(0,  false, true))
            .unwrap();

        let remote_actor_count = context.system.get_available_actor_count_for_pool(&context.actor_ref.get_address().pool).unwrap();
        let mut actors = Vec::new();
        for i in 0..remote_actor_count {
            let actor = context.system
                .builder()
                .set_pool_name(&context.actor_ref.get_address().pool)
                .spawn(format!("{}-tcp-{}", context.actor_ref.get_address().actor, i), TcpRemoteActorFactory::new())
                .unwrap();

            actors.push(actor.clone());
            router.send(AddActorMessage::new(actor)).unwrap();
        }

        Self {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            router,
            actors,
            remote_actor_count,
            graceful_shutdown_time_in_seconds,
        }
    }
}
impl Actor for NetworkManager {
    fn pre_stop(&mut self, _context: &ActorContext<Self>) {
        println!("PRE STOP");
        sleep(Duration::from_secs(self.graceful_shutdown_time_in_seconds as u64));
        println!("manager -> is_stopping =true");
        self.is_stopping.store(true, Ordering::Relaxed);

    }
    fn post_stop(&mut self, _context: &ActorContext<Self>) {
        let _ = self.router.stop();
        for actor in &self.actors {
            let _ = actor.stop();
        }
        for actor in &self.actors {
            actor.wait_for_stop();
        }

        println!("Actors have been stopped!");
        self.is_stopped.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect("127.0.0.1:2022");
    }
}

// define a factory that creates the `Actor` for us
pub struct NetworkManagerFactory {
    graceful_shutdown_time_in_seconds: u64
}
impl NetworkManagerFactory {
    pub fn new(graceful_shutdown_time_in_seconds: u64) -> Self {
        Self {
            graceful_shutdown_time_in_seconds
        }
    }
}
impl ActorFactory<NetworkManager> for NetworkManagerFactory {
    fn new_actor(
        &mut self,
        context: ActorContext<NetworkManager>,
    ) -> Result<NetworkManager, Box<dyn Error>> {
        Ok(NetworkManager::new(context, self.graceful_shutdown_time_in_seconds))
    }
}

// implement our message for the `Actor`
impl Handler<ActorInitMessage> for NetworkManager {
    fn handle(
        &mut self,
        _msg: ActorInitMessage,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {

        let is_stopped = self.is_stopped.clone();
        let is_stopping = self.is_stopping.clone();
        let remote_actor_count = self.remote_actor_count;

        let listener = TcpListener::bind("127.0.0.1:2022").unwrap();
        let router = self.router.clone();
            thread::spawn(move || {
                let pool = ThreadPool::new(remote_actor_count);
                for stream in listener.incoming() {
                    if is_stopped.load(Ordering::Relaxed) {
                        println!("NOW THE THREADS DIE!;");
                        pool.join();
                        println!("ALL DEAD!");
                        return;
                    }
                    if is_stopping.load(Ordering::Relaxed) {
                        continue;
                    }

                    match stream {
                        Ok(mut stream) => {
                            let router = router.clone();
                            pool.execute(move || {
                                let buf_reader = BufReader::new(&mut stream);
                                //let http_req = String::new();
                                //let http_request = buf_reader.read_line(&mut http_req);
                                let __http_request: Vec<_> = buf_reader
                                    .lines()
                                    .map(|result| result.unwrap())
                                    .take_while(|line| !line.is_empty())
                                    .collect();

                                router.send(NetworkMessage {
                                    id: 1,
                                    stream,
                                    content: SerializedMessage::new(Vec::new()),
                                }).unwrap();
                            })
                        },
                        Err(_e) => {
                            println!("SERS");
                        },
                    }
                }
            });


        Ok(ActorResult::Ok)
    }
}