use crate::net::net_messages::{
    AddTcpConnection, AddUdpSocket, ReceiveTcpMessage, ReceiveUdpMessage, RemoveTcpConnection,
};
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorInitMessage, ActorResult, ActorWrapper, Handler, NetConfig, NetConnectionType, NetProtocol};
use crate::router::Router;
use io_arc::IoArc;
use log::{debug, error, warn};
use mio::event::Source;
use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::net::Shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct NetManager<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,
{
    graceful_shutdown_time_in_seconds: Duration,
    on_stop_udp_timeout: Duration,
    router: ActorWrapper<R>,
    workers: Vec<ActorWrapper<T>>,
    net_configs: Vec<NetConfig>,
    is_stopping: Arc<AtomicBool>,
    is_stopped: Arc<AtomicBool>,
}

impl<T, R> NetManager<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,
{
    pub fn new(
        net_configs: Vec<NetConfig>,
        graceful_shutdown_time_in_seconds: Duration,
        on_stop_udp_timeout: Duration,
        workers: Vec<ActorWrapper<T>>,
        router: ActorWrapper<R>,
    ) -> Self
    {
        let is_stopping = Arc::new(AtomicBool::new(false));
        let is_stopped = Arc::new(AtomicBool::new(false));

        return Self {
            graceful_shutdown_time_in_seconds,
            on_stop_udp_timeout,
            router,
            workers,
            net_configs,
            is_stopping,
            is_stopped,
        };
    }
}
impl<T, R> Actor for NetManager<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,

{
    fn pre_stop(&mut self, _context: &ActorContext<Self>) {
        let iterations = 10;
        let iterate_graceful_stop = self.graceful_shutdown_time_in_seconds / iterations;

        sleep(iterate_graceful_stop);

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
                        let sock = UdpSocket::bind("127.0.0.1:0".parse().unwrap());
                        if sock.is_ok() {
                            let sock = sock.unwrap();
                            let _ = sock.send_to(b"", address.parse().unwrap());
                        }
                        break;
                    }
                }
            }
            if self.is_stopped.load(Ordering::Relaxed) {
                return;
            }
            sleep(iterate_graceful_stop);
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
                    let sock = UdpSocket::bind("127.0.0.1:0".parse().unwrap());
                    if sock.is_ok() {
                        let sock = sock.unwrap();
                        let _ = sock.send_to(b"", address.parse().unwrap());
                    }
                    break;
                }
            }
        }
    }
}

pub struct NetManagerFactory<T, R>
where

    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,
{
    net_configs: Vec<NetConfig>,
    graceful_shutdown_time_in_seconds: Duration,
    on_stop_udp_timeout: Duration,
    workers: Vec<ActorWrapper<T>>,
    router: ActorWrapper<R>,
    phantom: PhantomData<R>
}

impl<T, R> NetManagerFactory<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,
{
    pub fn new(
        net_configs: Vec<NetConfig>,
        graceful_shutdown_time_in_seconds: Duration,
        on_stop_udp_timeout: Duration,
        workers: Vec<ActorWrapper<T>>,
        router: ActorWrapper<R>,
    ) -> Self {
        return Self {
            net_configs,
            graceful_shutdown_time_in_seconds,
            on_stop_udp_timeout,
            workers,
            router,
            phantom: PhantomData,
        };
    }
}
impl<T, R> ActorFactory<NetManager<T, R>> for NetManagerFactory<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,
{
    fn new_actor(
        &mut self,
        context: ActorContext<NetManager<T, R>>,
    ) -> Result<NetManager<T, R>, Box<dyn Error>> {
        context.actor_ref.send(ActorInitMessage::new()).unwrap();
        return Ok(NetManager::new(
            self.net_configs.clone(),
            self.graceful_shutdown_time_in_seconds,
            self.on_stop_udp_timeout,
            self.workers.clone(),
            self.router.clone(),
        ));
    }
}

impl<T, R> Handler<ActorInitMessage> for NetManager<T, R>
where
    T: Handler<AddUdpSocket>
        + Handler<ReceiveUdpMessage>
        + Handler<AddTcpConnection>
        + Handler<RemoveTcpConnection>
        + Handler<ReceiveTcpMessage>
        + 'static,
    R: Router<T> + Handler<AddUdpSocket> + Handler<ReceiveUdpMessage> + Handler<AddTcpConnection> + Handler<RemoveTcpConnection> + Handler<ReceiveTcpMessage>,

{
    fn handle(
        &mut self,
        _msg: ActorInitMessage,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        let router = self.router.clone();
        let is_stopping = self.is_stopping.clone();
        let is_stopped = self.is_stopped.clone();
        let mut net_configs = self.net_configs.clone();
        let mut last_udp_message_received = Instant::now();
        let on_stop_udp_timeout = self.on_stop_udp_timeout.clone();
        let context = context.clone();
        thread::spawn(move || {
            let mut tcp_listeners: HashMap<Token, TcpListener> = HashMap::new();
            let mut udp_sockets: HashMap<Token, IoArc<UdpSocket>> = HashMap::new();
            let poll = Poll::new();
            if poll.is_err() {
                error!("Can't start Poll Port: {:?}", poll.err());
                is_stopped.store(true, Ordering::Relaxed);
                let _ = context.actor_ref.stop();
                return;
            }
            let mut poll = poll.unwrap();

            let mut i = 0;
            net_configs.sort_by_key(|c| c.protocol);
            for net_config in &net_configs {
                if net_config.connection_type == NetConnectionType::CLIENT {
                    println!("TEST: {:?}", net_config);
                    continue;
                }
                let token = Token(i);
                i += 1;

                let address = format!("{}:{}", net_config.host, net_config.port)
                    .parse()
                    .unwrap();

                match net_config.protocol {
                    NetProtocol::TCP => {
                        let listener = TcpListener::bind(address);
                        if listener.is_err() {
                            error!("Can't open TCP Port: {:?}", listener.err());
                            is_stopped.store(true, Ordering::Relaxed);
                            let _ = context.actor_ref.stop();
                            return;
                        }
                        let mut listener = listener.unwrap();
                        let res =
                            poll.registry()
                                .register(&mut listener, token, Interest::READABLE);
                        if res.is_err() {
                            error!("Can't register TCP listener: {:?}", res.err());
                            is_stopped.store(true, Ordering::Relaxed);
                            let _ = context.actor_ref.stop();
                            return;
                        }
                        tcp_listeners.insert(token, listener);
                    }
                    NetProtocol::UDP => {
                        let socket = UdpSocket::bind(address);
                        if socket.is_err() {
                            error!("Can't open TCP Port: {:?}", socket.err());
                            is_stopped.store(true, Ordering::Relaxed);
                            let _ = context.actor_ref.stop();
                            return;
                        }
                        let mut socket = socket.unwrap();
                        let res = poll
                            .registry()
                            .register(&mut socket, token, Interest::READABLE);
                        if res.is_err() {
                            error!("Can't register UDP Socket: {:?}", res.err());
                            is_stopped.store(true, Ordering::Relaxed);
                            let _ = context.actor_ref.stop();
                            return;
                        }
                        let socket = IoArc::new(socket);
                        udp_sockets.insert(token, socket.clone());
                        let _ = router.send(AddUdpSocket::new(token.0, socket));
                    }
                }
            }
            let num_tcp_listeners = tcp_listeners.len();
            let num_total_listeners = net_configs.len();

            let mut events = Events::with_capacity(1024);
            let mut streams = HashMap::new();

            let mut buf = [0; 65535];
            loop {
                if is_stopped.load(Ordering::Relaxed) {
                    return;
                }

                let res = poll.poll(&mut events, None);
                if res.is_err() {
                    debug!("Can't poll Network Events");
                    continue;
                }

                for event in events.iter() {
                    let stopping = is_stopping.load(Ordering::Relaxed);
                    if stopping
                        && streams.len() == 0
                        && last_udp_message_received.elapsed() > on_stop_udp_timeout
                    {
                        is_stopped.store(true, Ordering::Relaxed);
                        break;
                    }
                    let token = &event.token();
                    if token.0 < num_tcp_listeners {
                        let listener = tcp_listeners.get(token);
                        if listener.is_none() {
                            warn!("Can't find TcpListener for {:?}", token);
                            continue;
                        }
                        let listener = listener.unwrap();

                        loop {
                            match listener.accept() {
                                Ok((mut socket, address)) => {
                                    if stopping {
                                        let _ = socket.shutdown(Shutdown::Both);
                                        continue;
                                    }
                                    let res = socket.register(
                                        poll.registry(),
                                        Token(i),
                                        Interest::READABLE,
                                    );
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
                    } else if token.0 < num_total_listeners {
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
                            }
                        };
                        let request = String::from_utf8_lossy(&buf[..len]);
                        let _ = router.send(ReceiveUdpMessage::new(
                            token.0,
                            from,
                            request.into_owned(),
                        ));
                        last_udp_message_received = Instant::now();
                    } else {
                        if event.is_read_closed() || event.is_write_closed() {
                            let _ = streams.remove(&token.0);
                            let _ = router.send(RemoveTcpConnection::new(token.0));
                        } else if event.is_readable() {
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
                                .map(|result| match result {
                                    Ok(res) => {
                                        return res;
                                    }
                                    Err(err) => {
                                        warn!("Could not read from stream: {:?}", err);
                                        return String::from("");
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
