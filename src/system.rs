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
    is_running: Arc<AtomicBool>,
    config: TractorConfig,
    thread_pools: Arc<DashMap<String, usize>>,
    //// we need a threadsafe multi assoc dashmap where the key is the ID/Name of the actor system
    //// this dash map is required to have multiple different actorsystems, that might not run in the same process (aka cluster)
    //// the inner dashmap uses the ID/Name of the actual Actor as key
    //// value is then the wrapper that holds the sender
    //// this is used to send messages between actors by Name/ID
    //wip: Arc<DashMap<String, DashMap<String, Sender<Arc<dyn MessageTrait>>>>,
    //// we need a threadsafe DashMap where the key is the Name of the threadpool
    //// the value needs to be a Receiver, that stores the Actor as well as the corresponding Sender
    //// threadpools select their entry in the dashmap
    //// then pull an Actor from the Receiver, pull the next Message (non-blocking) from the Mailbox and execute the handler
    //// lastly the actor is pushed into the Sender again, so that it can be pulled again
    //wip: Arc<DashMap<String, Arc<Receiver<Arc<dyn ActorTrait>>>>>
    //one sender+receiver pair per Worker Pool
    //one sender+receiver pair per Actor
    sender: Sender<Arc<dyn ActorTrait>>,
    receiver: Receiver<Arc<dyn ActorTrait>>,
}

impl ActorSystem {
    pub fn new(config: TractorConfig) -> Self {
        let (sender, receiver) = unbounded();
        let thread_pools :Arc<DashMap<String, usize>> = Arc::new(DashMap::new());
        thread_pools.insert(String::from("system"), config.actor.system_thread_pool_size);
        thread_pools.insert(String::from("default"), config.actor.default_thread_pool_size);
        //let thread_pools :Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        //thread_pools.insert(String::from("system"), String::from("Arc::new(ThreadPool::new(config.actor.system_thread_pool_size))"));
        //thread_pools.insert(String::from("default"), String::from("Arc::new(ThreadPool::new(config.actor.thread_pool_size))"));
        ActorSystem {
            is_running: Arc::new(AtomicBool::new(true)),
            config,
            thread_pools,
            sender,
            receiver,
        }

    }

    pub fn add_pool(&self, name: &str, threads: usize) {
        if !self.thread_pools.contains_key(name) {
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
                        loop {
                            println!("I AM: {}-{}", pool_name, i);

                            sleep(Duration::from_secs((1) as u64));
                            if !system.thread_pools.contains_key("sers") {
                                system.thread_pools.insert(String::from("SERS"), 3);
                            }

                            //println!("I AM: {}-{}", key.clone(), key.clone());
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

    pub fn await_shutdown(&self) {
        while self.is_running.load(Ordering::Relaxed) {
            println!("I'm working here");
            sleep(Duration::from_secs(1));

        }
        println!("system_thread_pool_size: {}", self.config.actor.system_thread_pool_size);
        println!("thread_pool_size: {}", self.config.actor.default_thread_pool_size);

    }
}

