use std::collections::HashMap;
use threadpool::ThreadPool;
use std::time::Duration;
use crate::actor::actor_state::ActorState;
use std::thread::sleep;
use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use crate::config::pool_config::ThreadPoolConfig;
use crossbeam_channel::{Sender, Receiver, unbounded, bounded};
use crate::actor::executor::ExecutorTrait;
use crate::system::system_state::SystemState;
use crate::system::wakeup_manager::WakeupManager;

#[derive(Clone)]
pub struct ThreadPoolManager {
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
}

impl ThreadPoolManager {
    pub fn new() -> Self {
        Self {
            thread_pools: Arc::new(DashMap::new()),
        }
    }

    pub fn get_pool_sender(&self, name: &str) -> Sender<Arc<RwLock<dyn ExecutorTrait>>> {
        let pool = self.thread_pools.get(name).unwrap();
        let (_, sender, _) = pool.value().clone();
        sender
    }

    pub fn add_pool_with_config(&self, name: &str, thread_pool_config: ThreadPoolConfig) {
        if !self.thread_pools.contains_key(name) {
            let (sender, receiver) = if thread_pool_config.actor_limit == 0 {
                unbounded()
            } else {
                bounded(thread_pool_config.actor_limit)
            };
            self.thread_pools.insert(String::from(name), (thread_pool_config, sender, receiver));
        }
    }

    pub fn manage(&self, system_state: SystemState, wakeup_manager: WakeupManager) {
        let mut pools: HashMap<String, ThreadPool> = HashMap::new();
        loop {
            let is_stopped = system_state.is_stopped();
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
                    let pool_name = pool_name.clone();
                    let recv_timeout = Duration::from_secs(1);
                    let system_state = system_state.clone();
                    let wakeup_manager = wakeup_manager.clone();
                    pools.get(&pool_name).unwrap().execute(move || loop {
                        let is_system_stopping = system_state.is_stopping();
                        let mut actor_state = ActorState::Running;
                        let msg = receiver.recv_timeout(recv_timeout);
                        if msg.is_err() {
                            if system_state.is_stopped() {
                                return;
                            }
                            continue;
                        }
                        let ar = msg.unwrap();
                        {
                            let mut actor_ref = ar.write().unwrap();
                            let actor_config = actor_ref.get_config();
                            for _j in 0..actor_config.message_throughput {
                                actor_state = actor_ref.handle(is_system_stopping);
                                if actor_state != ActorState::Running {
                                    break;
                                }
                            }
                        };

                        if actor_state == ActorState::Running {
                            sender.send(ar).unwrap();
                        } else {
                            let address;
                            {
                                let actor_ref = ar.write().unwrap();
                                address = actor_ref.get_address();
                            }
                            if actor_state == ActorState::Sleeping {
                                wakeup_manager.add_sleeping_actor(address, ar);
                            } else {
                                println!("Actor has been stopped");
                                system_state.remove_actor(&address);
                            }
                        }
                    });
                }
            }
            sleep(Duration::from_secs((1) as u64));
        }
    }
}

