use crate::context::Context;
use crate::config::TractorConfig;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread::sleep;
use crate::actor::{ActorTrait, Handler, HelloWorld};
use std::borrow::Borrow;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crossbeam_channel::{unbounded, Receiver, Sender, bounded};
use crate::message::MessageTrait;
use std::sync::atomic::{AtomicBool, Ordering};
use dashmap::{DashMap};
use std::collections::HashMap;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorAddress {
    remote: String,
    system: String,
    pool: String,
    actor: String,
}

#[derive(Clone)]
pub struct ActorBuilder {
    name: String,
    pool: String,
    mailbox_size: usize,
    actor: Arc<dyn ActorTrait>,

}

impl ActorBuilder {
    pub fn build(&self) {

    }
}

#[derive(Clone)]
pub struct ActorSystem {
    name: String,
    is_running: Arc<AtomicBool>,
    config: TractorConfig,
    thread_pools: Arc<DashMap<String, usize>>,
    actor_queue: Arc<DashMap<ActorAddress, (Sender<String>, Receiver<String>)>>,
    //actor_mailboxes: Arc<DashMap<ActorAddress, (Sender<String>, Receiver<String>)>>,
}

impl ActorSystem {
    pub fn new(config: TractorConfig) -> Self {
        let thread_pools :Arc<DashMap<String, usize>> = Arc::new(DashMap::new());

        let actor_queue = Arc::new(DashMap::new());
        let system = ActorSystem {
            name: config.actor.name.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
            config,
            thread_pools,
            actor_queue
        };
        system.add_pool("system", system.config.actor.system_thread_pool_size);
        system.add_pool("default", system.config.actor.default_thread_pool_size);

        system

    }

    pub fn add_pool(&self, name: &str, threads: usize) {
        if !self.thread_pools.contains_key(name) {
            let (sender, receiver) = unbounded();
            let address = ActorAddress {
                system: self.name.clone(),
                pool: String::from(name),
                actor: String::from(""),
                remote: String::from("local")
            };
            self.actor_queue.insert(address, (sender, receiver));
            self.thread_pools.insert(String::from(name), threads);
        }
    }

    pub fn start(&self)  {
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
                if !pools.contains_key(&key) {
                    pools.insert(key.clone(), ThreadPool::new(pool.value().clone()));
                }
                let current = pools.get(&key).unwrap();
                for i in current.active_count()..current.max_count() {
                    let system = self.clone();
                    let pool_name = key.clone();
                    pools.get(&key).unwrap().execute(move || {
                        let address = ActorAddress {
                            system: system.name.clone(),
                            pool: pool_name.clone(),
                            actor: String::from(""),
                            remote: String::from("local")
                        };
                        let tuple = system.actor_queue.get(&address).unwrap();
                        let (sender, receiver) = tuple.value();
                        loop {
                            //test start
                            if !system.thread_pools.contains_key("sers") {
                                system.add_pool("sers", 16);

                                let sers_address = ActorAddress {
                                    system: system.name.clone(),
                                    pool: String::from("sers"),
                                    actor: String::from(""),
                                    remote: String::from("local")
                                };
                                let tuple = system.actor_queue.get(&sers_address).unwrap();
                                let (sender, _) = tuple.value();


                                sender.send(String::from("A1"));
                                sender.send(String::from("A2"));
                                sender.send(String::from("A3"));
                                sender.send(String::from("A4"));
                                sender.send(String::from("A5"));
                                sender.send(String::from("A6"));

                                sender.send(String::from("B1"));
                                sender.send(String::from("B2"));
                                sender.send(String::from("B3"));
                                sender.send(String::from("B4"));
                                sender.send(String::from("B5"));
                                sender.send(String::from("B6"));

                                sender.send(String::from("C1"));
                                sender.send(String::from("C2"));
                                sender.send(String::from("C3"));
                                sender.send(String::from("C4"));
                                sender.send(String::from("C5"));
                                sender.send(String::from("C6"));

                            }
                            //test end

                            let actor_ref = receiver.recv().unwrap();
                            println!("{}-{}-{}-{}: is working", system.name, pool_name, actor_ref, i);

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

    pub fn actor_of(&self, name: String, actor: impl ActorTrait + 'static) {
        self.spawn(ActorBuilder{
            name,
            actor: Arc::new(actor),
            pool: String::from("default"),
            mailbox_size: 0
        });
    }

    pub fn spawn(&self, builder: ActorBuilder) {
        let sender :Sender<String>;
        let receiver :Receiver<String>;
        if builder.mailbox_size == 0 {
            let (s, r)= unbounded();
            sender = s;
            receiver = r;
        }
        else {
            let (s, r)= bounded(builder.mailbox_size);
            sender = s;
            receiver = r;
        }
        sender.send(String::from("sers"));


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

