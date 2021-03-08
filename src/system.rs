use crate::actor::{ActorAddress, ActorTrait, Handler};
use crate::actor_config::ActorConfig;
use crate::actor_ref::{ActorRef, ActorRefTrait};
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
use std::time::Duration;
use threadpool::ThreadPool;
use std::panic;
use std::panic::UnwindSafe;
use crate::prelude::{ActorState};
use crossbeam_utils::atomic::AtomicCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ActorSystem {
    name: String,
    is_running: Arc<AtomicBool>,
    config: TyractorsaurConfig,
    thread_pools: Arc<
        DashMap<
            String,
            (
                ThreadPoolConfig,
                Sender<AtomicCell<Box<dyn ActorRefTrait>>>,
                Receiver<AtomicCell<Box<dyn ActorRefTrait>>>,
            ),
        >,
    >,
    actor_mailboxes: Arc<DashMap<ActorAddress, (Sender<String>, Receiver<String>)>>,
}

impl ActorSystem {
    pub fn new(config: TyractorsaurConfig) -> Self {
        let thread_pools = Arc::new(DashMap::new());
        let actor_mailboxes = Arc::new(DashMap::new());
        let thread_pool_config = config.thread_pool.clone();
        let system = ActorSystem {
            name: config.global.name.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
            config,
            thread_pools,
            actor_mailboxes,
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
        self.start_system_actors();
        println!("???");
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
                            let mut actor_ref = ar.into_inner();
                            let actor_config = actor_ref.get_config();
                            for j in 0..actor_config.message_throughput {
                                actor_ref.handle();
                                actor_state = actor_ref.get_current_state();
                                if actor_state != ActorState::Running {
                                    break;
                                }
                            }


                        //wip: sleep management is currently missing, but at least actors can be stopped
                        if actor_state != ActorState::Stopped {
                            sender.send(AtomicCell::new(actor_ref));
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
        let actor_ref = ActorRef::new(actor, actor_config, sender, receiver);
        let (_, sender, _) = tuple.value();
        let abc = actor_ref.clone();
        sender.send(AtomicCell::new(Box::new(abc)));
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
}
