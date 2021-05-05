use crate::actor::actor_address::ActorAddress;
use crate::actor::executor::ExecutorTrait;
use crate::system::system_state::SystemState;
use crate::system::thread_pool_manager::ThreadPoolManager;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct Wakeup {
    pub iteration: usize,
    pub actor_address: ActorAddress,
}

#[derive(Clone)]
pub struct WakeupManager {
    sleeping_actors: Arc<DashMap<ActorAddress, Arc<RwLock<dyn ExecutorTrait>>>>,
    wakeup_queue_in: Sender<Wakeup>,
    wakeup_queue_out: Receiver<Wakeup>,
}

impl WakeupManager {
    pub fn new() -> Self {
        let (wakeup_queue_in, wakeup_queue_out) = unbounded();
        Self {
            sleeping_actors: Arc::new(DashMap::new()),
            wakeup_queue_in,
            wakeup_queue_out,
        }
    }

    pub fn add_sleeping_actor(&self, address: ActorAddress, actor: Arc<RwLock<dyn ExecutorTrait>>) {
        self.sleeping_actors.insert(address, actor);
    }

    pub fn wakeup(&self, address: ActorAddress) {
        self.wakeup_queue_in
            .send(Wakeup {
                actor_address: address,
                iteration: 0,
            })
            .unwrap();
    }

    pub fn manage(&self, system_status: SystemState, thread_pool_manager: ThreadPoolManager) {
        let mut wake_deduplication: HashMap<ActorAddress, Instant> = HashMap::new();
        let recv_timeout = Duration::from_secs(1);
        loop {
            if system_status.is_stopped() {
                return;
            }
            if system_status.is_stopping() {
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
                    let sender = thread_pool_manager.get_pool_sender(&pool_name);
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
                    .send(Wakeup {
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
            let sender = thread_pool_manager.get_pool_sender(&wakeup_message.actor_address.pool);
            sender.send(actor_ref).unwrap();
        }
    }
}
