use crate::context::Context;
use crate::config::TractorConfig;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread::sleep;
use crate::actor::{ActorTrait, Handler, ActorAddress};
use std::borrow::Borrow;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use crossbeam_channel::{unbounded, Receiver, Sender, bounded};
use crate::message::MessageTrait;
use std::sync::atomic::{AtomicBool, Ordering};
use dashmap::{DashMap};
use std::collections::HashMap;
use crate::builder::ActorBuilder;
use crate::actor_ref::{ActorRef, ActorRefTrait};

pub const DEFAULT_POOL: &str = "default";
pub const SYSTEM_POOL: &str = "system";




#[derive(Clone)]
pub struct ActorSystem {
    name: String,
    is_running: Arc<AtomicBool>,
    config: TractorConfig,
    thread_pools: Arc<DashMap<String, (usize, Sender<Arc<dyn ActorRefTrait>>, Receiver<Arc<dyn ActorRefTrait>>)>>,
    actor_mailboxes: Arc<DashMap<ActorAddress, (Sender<String>, Receiver<String>)>>,
}

impl ActorSystem {
    pub fn new(config: TractorConfig) -> Self {
        let thread_pools = Arc::new(DashMap::new());
        let actor_mailboxes = Arc::new(DashMap::new());
        let system = ActorSystem {
            name: config.actor.name.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
            config,
            thread_pools,
            actor_mailboxes
        };
        system.add_pool(SYSTEM_POOL, system.config.actor.system_thread_pool_size);
        system.add_pool(DEFAULT_POOL, system.config.actor.default_thread_pool_size);
        system.start();
        system

    }

    pub fn add_pool(&self, name: &str, threads: usize) {
        if !self.thread_pools.contains_key(name) {
            let (sender, receiver) = unbounded();
            self.thread_pools.insert(String::from(name), (threads, sender, receiver));
        }
    }

    fn start(&self)  {
        let background_pool = ThreadPool::new(1);
        let s = self.clone();
        background_pool.execute(move || s.manage_threads());
        self.start_system_actors();
        println!("???");

    }

    fn manage_threads(&self) {
        let mut pools :HashMap<String, ThreadPool> = HashMap::new();

        loop {
            for pool in self.thread_pools.iter() {
                let key = pool.key().clone();
                let (pool_size, pool_sender, pool_receiver) = pool.value().clone();
                if !pools.contains_key(&key) {
                    pools.insert(key.clone(), ThreadPool::new(pool_size.clone()));
                }
                let current = pools.get(&key).unwrap();
                for i in current.active_count()..current.max_count() {
                    let sender = pool_sender.clone();
                    let receiver = pool_receiver.clone();
                    let system = self.clone();
                    let pool_name = key.clone();
                    pools.get(&key).unwrap().execute(move || {
                        loop {
                            //test start
                            if !system.thread_pools.contains_key("sers") {
                                system.add_pool("sers", 16);

                                let tuple = system.thread_pools.get("sers").unwrap();
                                let (_, sender, _) = tuple.value();


                                //sender.send(String::from("A1"));
                                //sender.send(String::from("A2"));
                                //sender.send(String::from("A3"));
                                //sender.send(String::from("A4"));
                                //sender.send(String::from("A5"));
                                //sender.send(String::from("A6"));
                                //sender.send(String::from("B1"));
                                //sender.send(String::from("B2"));
                                //sender.send(String::from("B3"));
                                //sender.send(String::from("B4"));
                                //sender.send(String::from("B5"));
                                //sender.send(String::from("B6"));
                                //sender.send(String::from("C1"));
                                //sender.send(String::from("C2"));
                                //sender.send(String::from("C3"));
                                //sender.send(String::from("C4"));
                                //sender.send(String::from("C5"));
                                //sender.send(String::from("C6"));

                            }
                            //test end

                            let mut actor_ref = receiver.recv().unwrap();
                            let a = actor_ref.get_actor();

                            //if actor_ref == "hello-world" {
                            //    println!("{}-{}-{}-{}: is working", system.name, pool_name, actor_ref, i);
                            //}

                            sender.send(actor_ref);
                            //system.is_running.swap(false, Ordering::Relaxed);
                        }
                    });
                }
            }
            sleep(Duration::from_secs((1) as u64));
        }
    }

    fn start_system_actors(&self) {

    }

    pub fn builder(&self, name: impl Into<String>) -> ActorBuilder {
        ActorBuilder::new(self.clone(), name.into())
    }

    pub fn spawn<A>(&self, name: String, actor: A, mailbox_size: usize, pool: &str) -> ActorRef<A>
    where
        A: ActorTrait + Clone + 'static,
    {
        let (sender, receiver) = unbounded();

        let actor_ref = ActorRef::new(Arc::new(RwLock::new(actor)), sender, receiver);
        let tuple = self.thread_pools.get(pool).unwrap();
        let (_, sender, _) = tuple.value();
        let abc = actor_ref.clone();
        sender.send(Arc::new(abc));
        actor_ref

    }

    pub fn stop(&self) {
    }

    pub fn await_shutdown(&self) {
        while self.is_running.load(Ordering::Relaxed) {
            println!("I'm working here");
            sleep(Duration::from_secs(1));

        }
        self.stop();
        println!("system_thread_pool_size: {}", self.config.actor.system_thread_pool_size);
        println!("thread_pool_size: {}", self.config.actor.default_thread_pool_size);

    }
}

