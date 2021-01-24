use crate::context::Context;
use crate::config::TractorConfig;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread::sleep;

pub struct ActorSystem {
    config: TractorConfig
}

impl ActorSystem {
    pub fn new(config: TractorConfig) -> Self {
        ActorSystem {
            config
        }

    }

    pub fn start(&self) {
        let system_pool = ThreadPool::new(self.config.actor.system_thread_pool_size);
        let worker_pool = ThreadPool::new(self.config.actor.thread_pool_size);
        let key = "sers";
        loop {
            for i in 0..(system_pool.max_count() - system_pool.active_count()) {
                system_pool.execute(move || {
                    loop {
                        println!("Hi I'm System-{}", key);
                        sleep(Duration::from_secs((i + 1) as u64));
                    }
                })
            }
            for i in 0..(worker_pool.max_count() - worker_pool.active_count()) {
                worker_pool.execute(move || {
                    loop {
                        println!("Hi I'm Worker-{}", key);
                        sleep(Duration::from_secs((i + 1) as u64));
                        panic!("broken")
                    }
                })
            }
            sleep(Duration::from_secs((1) as u64));

        }

    }

    pub fn await_shutdown(&self) {
        sleep(Duration::from_secs(60));
        println!("system_thread_pool_size: {}", self.config.actor.system_thread_pool_size);
        println!("thread_pool_size: {}", self.config.actor.thread_pool_size);

    }
}

