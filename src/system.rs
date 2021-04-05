use crate::actor::{ActorAddress, ActorTrait, Handler};
use crate::actor_config::ActorConfig;
use crate::actor_ref::{ActorHandler, ActorRefTrait};
use crate::builder::ActorBuilder;
use crate::config::prelude::*;
use crate::context::Context;
use crate::message::MessageTrait;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use std::panic;
use std::panic::UnwindSafe;
use crate::prelude::{ActorState, ActorRef, Mailbox};
use crossbeam_utils::atomic::AtomicCell;
use std::rc::Rc;

pub struct WakeupMessage {
    iteration: usize,
    actor_address: ActorAddress,
}

#[derive(Clone)]
pub struct ActorSystem {
    name: String,
    is_running: Arc<AtomicBool>,
    config: Arc<TyractorsaurConfig>,
    thread_pools: Arc<
        DashMap<
            String,
            (
                ThreadPoolConfig,
                Sender<Arc<RwLock<dyn ActorRefTrait>>>,
                Receiver<Arc<RwLock<dyn ActorRefTrait>>>,
            ),
        >,
    >,
    sleeping_actors: Arc<DashMap<ActorAddress, Arc<RwLock<dyn ActorRefTrait>>>>,
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
            name: config.global.name.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
            config: Arc::new(config),
            thread_pools,
            wakeup_queue_in,
            wakeup_queue_out,
            sleeping_actors
        };

        for (key, value) in thread_pool_config.config.iter() {
            system.add_pool_with_config(key, value.clone());
        }
        system.add_pool(SYSTEM_POOL);
        let system_pool_config = thread_pool_config.config.get(SYSTEM_POOL).unwrap();
        system.add_pool_with_config(SYSTEM_POOL, system_pool_config.clone());
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
        let background_pool = ThreadPool::with_name(String::from("background"), 1);
        let s = self.clone();
        background_pool.execute(move || s.manage_threads());
        let waker_pool = ThreadPool::with_name(String::from("waker"), 1);
        let s = self.clone();
        waker_pool.execute(move || s.wake());
        self.start_system_actors();
        println!("???");
    }

    fn wake(&self) {
        let mut wake_deduplication: HashMap<ActorAddress, Instant> = HashMap::new();

        loop {
            let wakeup_message= self.wakeup_queue_out.recv().unwrap();
            if wake_deduplication.contains_key(&wakeup_message.actor_address) && wakeup_message.iteration == 0 {
                // actors have a minimum uptime of 1 second
                // this ensures a guaranteed de-duplication of all wakeup calls to a single actor
                let last_wakeup = wake_deduplication.get(&wakeup_message.actor_address).unwrap();
                let duration = last_wakeup.elapsed();
                if duration >= Duration::from_millis(750) {
                    wake_deduplication.remove(&wakeup_message.actor_address);
                }
                else {
                    continue;
                }
            }

            wake_deduplication.insert(wakeup_message.actor_address.clone(), Instant::now());
            if !self.sleeping_actors.contains_key(&wakeup_message.actor_address) {
                self.wakeup_queue_in.send(WakeupMessage {
                    iteration: (wakeup_message.iteration + 1),
                    actor_address: wakeup_message.actor_address,
                }).unwrap();
                continue;
            }

            let actor_ref = self.sleeping_actors.remove(&wakeup_message.actor_address).unwrap().1;
            {
                let mut actor_ref = actor_ref.write().unwrap();
                actor_ref.wakeup();
            }
            let pool = self.thread_pools.get(&wakeup_message.actor_address.pool).unwrap();
            let (_, sender, _) = pool.value();
            sender.send(actor_ref).unwrap();
        }
    }

    fn manage_threads(&self) {
        let mut pools: HashMap<String, ThreadPool> = HashMap::new();

        loop {
            for pool in self.thread_pools.iter() {
                let pool_name = pool.key().clone();
                let (pool_config, pool_sender, pool_receiver) = pool.value().clone();
                if !pools.contains_key(&pool_name) {
                    pools.insert(
                        pool_name.clone(),
                        ThreadPool::with_name(pool_name.clone(), pool_config.thread_count.clone()),
                    );
                }
                let current = pools.get(&pool_name).unwrap();
                for i in current.active_count()..current.max_count() {
                    let sender = pool_sender.clone();
                    let receiver = pool_receiver.clone();
                    let system = self.clone();
                    let pool_name = pool_name.clone();
                    let pool_config = pool_config.clone();
                    pools.get(&pool_name).unwrap().execute(move || loop {
                        let mut actor_state = ActorState::Running;
                        let mut ar = receiver.recv().unwrap();
                        {
                            let mut actor_ref = ar.write().unwrap();
                            let actor_config = actor_ref.get_config();
                            for j in 0..actor_config.message_throughput {
                                actor_state = actor_ref.handle();
                                if actor_state != ActorState::Running {
                                    break;
                                }
                            }
                        };


                        if actor_state == ActorState::Running {
                            sender.send(ar).unwrap();
                        }
                        else if actor_state == ActorState::Sleeping {
                            let mut address = None;
                            {
                                let mut actor_ref = ar.write().unwrap();
                                address = Some(actor_ref.get_address());
                            }
                            system.sleeping_actors.insert(address.unwrap(), ar);
                        }
                        else {
                            println!("Actor has been stopped");
                        }
                    });
                }
            }
            sleep(Duration::from_secs((1) as u64));
        }
    }

    fn start_system_actors(&self) {}

    pub fn builder(&self, name: impl Into<String>) -> ActorBuilder {
        ActorBuilder::new(self.clone(), name.into())
    }

    pub fn spawn<A>(&self, actor: A, actor_config: ActorConfig) -> ActorRef<A>
    where
        A: ActorTrait + Clone + UnwindSafe + 'static,
    {
        let (sender, receiver) = if actor_config.mailbox_size == 0 {
            unbounded()
        } else {
            bounded(actor_config.mailbox_size)
        };

        let tuple = self.thread_pools.get(&actor_config.pool_name).unwrap();
        let mailbox = Mailbox {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            msg_in: sender,
        };
        let actor_address = ActorAddress{
            actor: actor_config.actor_name.clone(),
            system: self.name.clone(),
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };
        let actor_ref = ActorRef::new(mailbox.clone(), actor_address, self.clone());
        let actor_handler = ActorHandler::new(actor, actor_config, mailbox, receiver, self.clone(),self.name.clone(), actor_ref.clone());

        self.sleeping_actors.insert(actor_handler.get_address(), Arc::new(RwLock::new(actor_handler)));

        actor_ref

    }

    pub fn stop(&self) {}

    pub fn await_shutdown(&self) {
        while self.is_running.load(Ordering::Relaxed) {
            sleep(Duration::from_secs(1));
        }
        self.stop();
    }

    pub fn get_config(&self) -> &TyractorsaurConfig {
        &self.config
    }

    pub fn wakeup(&self, actor_address: ActorAddress)
    {
        self.wakeup_queue_in.send(WakeupMessage {
            iteration: 0,
            actor_address,
        }).unwrap();
    }
}
