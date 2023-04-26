use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::net::Shutdown;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use io_arc::IoArc;
use log::{error, warn};
use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token};
use mio::event::Source;
use crate::net::net_worker::{AddTcpConnection, AddUdpSocket, ReceiveTcpMessage, ReceiveUdpMessage, RemoveTcpConnection};
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorInitMessage, ActorResult, ActorWrapper, Handler, NetConfig, NetProtocol};
use crate::router::{AddActorMessage, ShardedRouter, ShardedRouterFactory};

pub struct NetManager<T>
    where
        T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    graceful_shutdown_time_in_seconds: usize,
    router: ActorWrapper<ShardedRouter<T>>,
    workers: Vec<ActorWrapper<T>>,
    net_configs: Vec<NetConfig>,
    is_stopping: Arc<AtomicBool>,
    is_stopped: Arc<AtomicBool>,


}

impl<T> NetManager<T>
    where

        T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    pub fn new<F>(context: ActorContext<Self>, net_configs: Vec<NetConfig>, graceful_shutdown_time_in_seconds: usize, worker_factory: F) -> Self
    where F: ActorFactory<T> + Clone + 'static, {

        let pool_name = &context.actor_ref.get_address().pool;

        let router = context.system.builder().set_pool_name(pool_name).spawn("net-least-message", ShardedRouterFactory::new( false, false)).unwrap();
        let worker_count = context.system.get_available_actor_count_for_pool(pool_name).unwrap();
        let workers = context.system.builder().set_pool_name(pool_name).spawn_multiple("net-worker", worker_factory.clone(), worker_count).unwrap();
        for worker in &workers {
            router.send(AddActorMessage::new(worker.clone())).unwrap();
        }

        let is_stopping = Arc::new(AtomicBool::new(false));
        let is_stopped = Arc::new(AtomicBool::new(false));

        return Self {
            graceful_shutdown_time_in_seconds,
            router,
            workers,
            net_configs,
            is_stopping,
            is_stopped,
        };
    }
}
impl<T> Actor for NetManager<T>
    where
        T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    fn pre_stop(&mut self, _context: &ActorContext<Self>) {
        if self.graceful_shutdown_time_in_seconds == 0 {
            self.is_stopping.store(true, Ordering::Relaxed);
            return;
        }
        let graceful_stop_in_millis = self.graceful_shutdown_time_in_seconds * 1000;
        let iterations = 10;
        let iterate_graceful_stop_in_millis = graceful_stop_in_millis / iterations;


        sleep(Duration::from_millis(iterate_graceful_stop_in_millis as u64));

        self.is_stopping.store(true, Ordering::Relaxed);

        for _ in 0..iterations {
            for net_config in &self.net_configs {
                let address = format!("{}:{}", net_config.host, net_config.port);
                match net_config.protocol {
                    NetProtocol::TCP => {
                        let _ = TcpStream::connect(address.parse().unwrap());
                        break;
                    }
                    NetProtocol::UDP => {
                        break;
                    }
                }
            }
            if self.is_stopped.load(Ordering::Relaxed) {
                return;
            }
            sleep(Duration::from_millis(iterate_graceful_stop_in_millis as u64));

        }
    }
    fn post_stop(&mut self, _context: &ActorContext<Self>) {
        let _ = self.router.stop();
        for worker in &self.workers {
            let _ = worker.stop();
        }
        for worker in &self.workers {
            worker.wait_for_stop();
        }

        self.is_stopped.store(true, Ordering::Relaxed);
        for net_config in &self.net_configs {
            let address = format!("{}:{}", net_config.host, net_config.port);
            match net_config.protocol {
                NetProtocol::TCP => {
                    let _ = TcpStream::connect(address.parse().unwrap());
                    break;
                }
                NetProtocol::UDP => {
                    break;
                }
            }
        }
    }
}

pub struct NetManagerFactory<F, T>
where
    F: ActorFactory<T> + Clone + 'static,
    T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    net_configs: Vec<NetConfig>,
    graceful_shutdown_time_in_seconds: usize,
    worker_factory: F,
    phantom: PhantomData<T>,
}

impl<F, T> NetManagerFactory<F, T>
where
    F: ActorFactory<T> + Clone + 'static,
    T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,

{
    pub fn new(net_configs: Vec<NetConfig>, graceful_shutdown_time_in_seconds: usize, worker_factory: F) -> Self {
        return Self {
            net_configs,
            graceful_shutdown_time_in_seconds,
            worker_factory,
            phantom: PhantomData,
        };
    }
}
impl<F, T> ActorFactory<NetManager<T>> for NetManagerFactory<F, T>
where
    F: ActorFactory<T> + Clone + 'static,
    T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    fn new_actor(&mut self, context: ActorContext<NetManager<T>>) -> Result<NetManager<T>, Box<dyn Error>> {
        context.actor_ref.send(ActorInitMessage::new()).unwrap();
        return Ok(NetManager::new(context, self.net_configs.clone(), self.graceful_shutdown_time_in_seconds, self.worker_factory.clone()));
    }
}

impl<T> Handler<ActorInitMessage> for NetManager<T>
    where
        T: Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage> + 'static,
{
    fn handle(&mut self, _msg: ActorInitMessage, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let router = self.router.clone();
        let is_stopping = self.is_stopping.clone();
        let is_stopped = self.is_stopped.clone();
        let mut net_configs = self.net_configs.clone();

        thread::spawn(move || {
            let mut tcp_listeners :HashMap<Token, TcpListener> = HashMap::new();
            let mut udp_sockets:HashMap<Token, IoArc<UdpSocket>> = HashMap::new();
            let mut poll = Poll::new().unwrap();

            let mut i = 0;
            net_configs.sort_by_key(|c| c.protocol);
            for net_config in &net_configs {
                let token = Token(i);
                i += 1;

                let address = format!("{}:{}", net_config.host, net_config.port).parse().unwrap();

                match net_config.protocol {
                    NetProtocol::TCP => {
                        let mut listener = TcpListener::bind(address).unwrap();
                        poll.registry().register(&mut listener, token, Interest::READABLE).unwrap();
                        tcp_listeners.insert(token, listener);
                    },
                    NetProtocol::UDP => {
                        let mut socket = UdpSocket::bind(address).unwrap();
                        poll.registry().register(&mut socket, token, Interest::READABLE).unwrap();
                        let socket = IoArc::new(socket);
                        udp_sockets.insert(token, socket.clone());
                        let _ = router.send(AddUdpSocket::new(token.0, socket));
                    }
                }
            }
            let num_tcp_listeners = tcp_listeners.len();
            let num_total_listeners = net_configs.len();


            let mut events = Events::with_capacity(1024);
            let mut streams =  HashMap::new();

            let mut buf = [0; 65535];
            loop {

                if is_stopped.load(Ordering::Relaxed) {
                    return;
                }

                poll.poll(&mut events, None).unwrap();

                for event in events.iter() {
                    let stopping = is_stopping.load(Ordering::Relaxed);
                    if stopping && streams.len() == 0  {
                        is_stopped.store(true, Ordering::Relaxed);
                        break;
                    }
                    let token = &event.token();
                    if token.0 < num_tcp_listeners {
                        let listener = tcp_listeners.get(token).unwrap();

                        loop {
                            match listener.accept() {
                                Ok((mut socket, address)) => {
                                    if stopping {
                                        let _ = socket.shutdown(Shutdown::Both);
                                        continue;
                                    }
                                    let res = socket.register(poll.registry(), Token(i), Interest::READABLE);
                                    if res.is_err() {
                                        error!("Could not register TcpStream. {:?}", res.err());
                                    }
                                    let sock = IoArc::new(socket);
                                    streams.insert(i, (sock.clone(), address.clone()));
                                    let _ = router.send(AddTcpConnection::new(i, sock, address));

                                    i += 1;
                                    if i < num_total_listeners {
                                        i = num_total_listeners;
                                    }
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    break;
                                }
                                Err(e) => {
                                    error!("Something went wrong with the Listener. {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                    else if token.0 < num_total_listeners {
                        //UDP handling
                        let socket = udp_sockets.get(&token);
                        if socket.is_none() {
                            error!("Something went wrong with the UDP Socket.");
                        }
                        let socket = socket.unwrap();

                        let (len, from) = match socket.as_ref().recv_from(&mut buf) {
                            Ok(v) => v,

                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::WouldBlock {
                                    continue;
                                }
                                panic!("recv() failed: {:?}", e);
                            },
                        };
                        let request = String::from_utf8_lossy(&buf[..len]);
                        let _ = router.send(ReceiveUdpMessage::new(token.0, from, request.into_owned()));

                    }
                    else {
                        if event.is_read_closed() || event.is_write_closed() {
                            let _ = streams.remove(&token.0);
                            let _ = router.send(RemoveTcpConnection::new(token.0));

                        }
                        else if event.is_readable() {
                            let stream = streams.get(&token.0);
                            if stream.is_none() {
                                let _ = streams.remove(&token.0);
                                let _ = router.send(RemoveTcpConnection::new(token.0));
                                continue;
                            }
                            let (stream, _address) = stream.unwrap();
                            let buf_reader = BufReader::new(stream.clone());
                            let request: Vec<String> = buf_reader
                                .lines()
                                .map(|result| {
                                    match result {
                                        Ok(res) => {
                                            return res;
                                        }
                                        Err(err) => {
                                            warn!("Could not read from stream: {:?}", err);
                                            return String::from("");
                                        }
                                    }
                                })
                                .take_while(|line| !line.is_empty())
                                .collect();
                            if !request.is_empty() {
                                let _ = router.send(ReceiveTcpMessage::new(token.0, request));
                            }

                        }
                    }
                }
            }
        });
        return Ok(ActorResult::Ok);
    }
}