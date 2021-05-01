use crate::actor::actor::Actor;
use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crate::config::tyractorsaur_config::{TyractorsaurConfig, DEFAULT_POOL};
use crate::config::pool_config::ThreadPoolConfig;
use crossbeam_channel::{Sender, Receiver, unbounded, bounded};
use crate::actor::executor::{ExecutorTrait, Executor};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use threadpool::ThreadPool;
use crate::actor::actor_state::ActorState;
use std::thread::sleep;
use crate::message::serialized_message::SerializedMessage;
use crate::actor::actor_builder::ActorBuilder;
use crate::actor::config::Config;
use crate::actor::actor_wrapper::ActorWrapper;
use std::panic::UnwindSafe;
use crate::actor::mailbox::Mailbox;
use crate::actor::context::Context;
use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_factory::ActorFactory;

pub struct WakeupMessage {
    iteration: usize,
    actor_address: ActorAddress,
}

#[derive(Clone)]
pub struct ActorSystem {
    actors: DashMap<ActorAddress, Arc<dyn Actor>>,
    total_actor_count: Arc<AtomicUsize>,
    name: String,
    is_forced_stop: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    is_stopped: Arc<AtomicBool>,
    config: Arc<TyractorsaurConfig>,
    thread_pools: Arc<
        DashMap<
            String,
            (
                ThreadPoolConfig,
                Sender<Arc<RwLock<dyn ExecutorTrait>>>,
                Receiver<Arc<RwLock<dyn ExecutorTrait>>>,
            ),
        >,
    >,
    sleeping_actors: Arc<DashMap<ActorAddress, Arc<RwLock<dyn ExecutorTrait>>>>,
    wakeup_queue_in: Sender<WakeupMessage>,
    wakeup_queue_out: Receiver<WakeupMessage>,
}

impl ActorSystem {
    pub fn new(config: TyractorsaurConfig) -> Self {
        let thread_pools = Arc::new(DashMap::new());
        let (wakeup_queue_in, wakeup_queue_out) = unbounded();
        let sleeping_actors = Arc::new(DashMap::new());
        let thread_pool_config = config.thread_pool.clone();
        let system = ActorSystem {
            actors: DashMap::new(),
            total_actor_count: Arc::new(AtomicUsize::new(0)),
            name: config.global.name.clone(),
            is_forced_stop: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            is_stopped: Arc::new(AtomicBool::new(false)),
            config: Arc::new(config.clone()),
            thread_pools,
            wakeup_queue_in,
            wakeup_queue_out,
            sleeping_actors,
        };

        for (key, value) in thread_pool_config.config.iter() {
            system.add_pool_with_config(key, value.clone());
        }
        system.start();
        system
    }

    pub fn add_pool(&self, name: &str) {
        let default_config = self.config.thread_pool.config.get(DEFAULT_POOL).unwrap();
        let config = self
            .config
            .thread_pool
            .config
            .get(name)
            .unwrap_or(default_config);
        self.add_pool_with_config(name, config.clone());
    }

    pub fn add_pool_with_config(&self, name: &str, thread_pool_config: ThreadPoolConfig) {
        if !self.thread_pools.contains_key(name) {
            let (sender, receiver) = if thread_pool_config.actor_limit == 0 {
                unbounded()
            } else {
                bounded(thread_pool_config.actor_limit)
            };
            self.thread_pools
                .insert(String::from(name), (thread_pool_config, sender, receiver));
        }
    }

    fn start(&self) {
        let s = self.clone();
        std::thread::spawn(move || s.manage_threads());
        let s = self.clone();
        std::thread::spawn(move || s.wake());
    }

    fn wake(&self) {
        let mut wake_deduplication: HashMap<ActorAddress, Instant> = HashMap::new();
        let recv_timeout = Duration::from_secs(1);
        loop {
            if self.is_stopped.load(Ordering::Relaxed) {
                return;
            }
            if self.is_stopping.load(Ordering::Relaxed) {
                let mut keys: Vec<ActorAddress> = Vec::new();
                for key in self.sleeping_actors.iter() {
                    keys.push(key.key().clone());
                }
                for key in keys {
                    let sleeping_actor = self.sleeping_actors.remove(&key).unwrap();
                    let pool_name = sleeping_actor.0.pool;
                    let actor_ref = sleeping_actor.1;
                    {
                        let mut actor_ref = actor_ref.write().unwrap();
                        actor_ref.wakeup();
                    }
                    let pool = self.thread_pools.get(&pool_name).unwrap();
                    let (_, sender, _) = pool.value();
                    sender.send(actor_ref).unwrap();
                }
                continue;
            }
            let msg = self.wakeup_queue_out.recv_timeout(recv_timeout);
            if msg.is_err() {
                continue;
            }
            let wakeup_message = msg.unwrap();

            if wake_deduplication.contains_key(&wakeup_message.actor_address)
                && wakeup_message.iteration == 0
            {
                // actors have a minimum uptime of 1 second
                // this ensures a guaranteed de-duplication of all wakeup calls to a single actor
                let last_wakeup = wake_deduplication
                    .get(&wakeup_message.actor_address)
                    .unwrap();
                let duration = last_wakeup.elapsed();
                if duration >= Duration::from_secs(4) {
                    wake_deduplication.remove(&wakeup_message.actor_address);
                } else {
                    continue;
                }
            }

            wake_deduplication.insert(wakeup_message.actor_address.clone(), Instant::now());
            if !self
                .sleeping_actors
                .contains_key(&wakeup_message.actor_address)
            {
                self.wakeup_queue_in
                    .send(WakeupMessage {
                        iteration: (wakeup_message.iteration + 1),
                        actor_address: wakeup_message.actor_address,
                    })
                    .unwrap();
                continue;
            }

            let actor_ref = self
                .sleeping_actors
                .remove(&wakeup_message.actor_address)
                .unwrap()
                .1;
            {
                let mut actor_ref = actor_ref.write().unwrap();
                actor_ref.wakeup();
            }
            let pool = self
                .thread_pools
                .get(&wakeup_message.actor_address.pool)
                .unwrap();
            let (_, sender, _) = pool.value();
            sender.send(actor_ref).unwrap();
        }
    }

    fn manage_threads(&self) {
        let mut pools: HashMap<String, ThreadPool> = HashMap::new();

        loop {
            let is_stopped = self.is_stopped.load(Ordering::Relaxed);
            if is_stopped {
                for pool in pools.iter() {
                    pool.1.join()
                }
                return;
            }
            for pool in self.thread_pools.iter() {
                let pool_name = pool.key().clone();
                let (pool_config, pool_sender, pool_receiver) = pool.value().clone();
                if !pools.contains_key(&pool_name) {
                    let thread_count = pool_config.threads_factor * num_cpus::get() as f32;
                    let mut thread_count = thread_count.floor() as usize;
                    if thread_count < pool_config.threads_min {
                        thread_count = pool_config.threads_min;
                    }
                    else if thread_count > pool_config.threads_max {
                        thread_count = pool_config.threads_max;
                    }

                    pools.insert(
                        pool_name.clone(),
                        ThreadPool::with_name(pool_name.clone(), thread_count),
                    );

                }
                let current = pools.get(&pool_name).unwrap();
                for _i in current.active_count()..current.max_count() {
                    let sender = pool_sender.clone();
                    let receiver = pool_receiver.clone();
                    let system = self.clone();
                    let pool_name = pool_name.clone();
                    let recv_timeout = Duration::from_secs(1);
                    pools.get(&pool_name).unwrap().execute(move || loop {
                        let system_is_stopping = system.is_stopping.load(Ordering::Relaxed);
                        let mut actor_state = ActorState::Running;
                        let msg = receiver.recv_timeout(recv_timeout);
                        if msg.is_err() {
                            if system.is_stopped.load(Ordering::Relaxed) {
                                return;
                            }
                            continue;
                        }
                        let ar = msg.unwrap();
                        {
                            let mut actor_ref = ar.write().unwrap();
                            let actor_config = actor_ref.get_config();
                            for _j in 0..actor_config.message_throughput {
                                actor_state = actor_ref.handle(system_is_stopping);
                                if actor_state != ActorState::Running {
                                    break;
                                }
                            }
                        };

                        if actor_state == ActorState::Running {
                            sender.send(ar).unwrap();
                        } else if actor_state == ActorState::Sleeping {
                            let address;
                            {
                                let actor_ref = ar.write().unwrap();
                                address = actor_ref.get_address();
                            }
                            system.sleeping_actors.insert(address, ar);
                        } else {
                            println!("Actor has been stopped");
                            system.total_actor_count.fetch_sub(1, Ordering::Relaxed);
                        }
                    });
                }
            }
            sleep(Duration::from_secs((1) as u64));
        }
    }

    pub fn send_to_address(&self, address: &ActorAddress, msg: SerializedMessage) {
        let target = self.actors.get(address);
        if target.is_some() {
            let target = target.unwrap();
            target.handle_serialized_message(msg);
        }
    }

    pub fn builder(&self, name: impl Into<String>) -> ActorBuilder {
        ActorBuilder::new(self.clone(), name.into())
    }

    pub fn spawn<A, P>(&self, actor_props: P, actor_config: Config) -> ActorWrapper<A>
    where
        A: Actor + UnwindSafe + 'static,
        P: ActorFactory<A> + 'static,
    {
        let (sender, receiver) = if actor_config.mailbox_size == 0 {
            unbounded()
        } else {
            bounded(actor_config.mailbox_size)
        };

        let mailbox = Mailbox {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            msg_in: sender,
        };
        let actor_address = ActorAddress {
            actor: actor_config.actor_name.clone(),
            system: self.name.clone(),
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };
        let actor_ref = ActorWrapper::new(mailbox.clone(), actor_address.clone(), self.clone());

        let context = Context {
            system: self.clone(),
            actor_ref: actor_ref.clone(),
        };
        let actor = actor_props.new_actor(context);
        let actor_handler = Executor::new(
            actor_props,
            actor_config,
            mailbox.clone(),
            receiver,
            self.clone(),
            self.name.clone(),
            actor_ref.clone(),
        );

        self.actors.insert(actor_address.clone(), Arc::new(actor));
        self.sleeping_actors.insert(
            actor_handler.get_address(),
            Arc::new(RwLock::new(actor_handler)),
        );

        self.total_actor_count.fetch_add(1, Ordering::Relaxed);
        actor_ref
    }

    pub fn remove_actor(&self, address: &ActorAddress) {
        self.actors.remove(address);
    }

    pub fn stop(&self, graceful_termination_timeout: Duration) {
        if self.is_stopping.load(Ordering::Relaxed) {
            return;
        }
        self.is_stopping.store(true, Ordering::Relaxed);
        let s = self.clone();
        std::thread::spawn(move || s.shutdown(graceful_termination_timeout));
    }

    fn shutdown(&self, timeout: Duration) {
        let now = Instant::now();
        while self.total_actor_count.load(Ordering::Relaxed) != 0 {
            if now.elapsed() >= timeout {
                self.is_forced_stop.store(true, Ordering::Relaxed);
                break;
            }
            sleep(Duration::from_secs(1));
        }
        self.actors.clear();
        self.is_stopped.store(true, Ordering::Relaxed)
    }

    pub fn await_shutdown(&self) -> i32 {
        while !self.is_stopped.load(Ordering::Relaxed) {
            sleep(Duration::from_secs(1));
        }
        self.is_forced_stop.load(Ordering::Relaxed) as i32
    }

    pub fn get_config(&self) -> &TyractorsaurConfig {
        &self.config
    }

    pub fn wakeup(&self, actor_address: ActorAddress) {
        self.wakeup_queue_in
            .send(WakeupMessage {
                iteration: 0,
                actor_address,
            })
            .unwrap();
    }
}
