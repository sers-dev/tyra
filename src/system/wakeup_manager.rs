use crate::actor::actor_address::ActorAddress;
use crate::actor::executor::ExecutorTrait;
use crate::system::system_state::SystemState;
use crate::system::thread_pool_manager::ThreadPoolManager;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Wakeup {
    pub iteration: usize,
    pub actor_address: ActorAddress,
}

pub struct Sleep {
    pub now: Instant,
    pub duration: Duration,
    pub actor_address: ActorAddress,
}

#[derive(Clone)]
pub struct WakeupManager {
    inactive_actors: Arc<DashMap<ActorAddress, Arc<RwLock<dyn ExecutorTrait>>>>,
    wakeup_queue_in: Sender<Wakeup>,
    wakeup_queue_out: Receiver<Wakeup>,
    sleep_queue_in: Sender<Sleep>,
    sleep_queue_out: Receiver<Sleep>,
}

impl WakeupManager {
    pub fn new() -> Self {
        let (wakeup_queue_in, wakeup_queue_out) = unbounded();
        let (sleep_queue_in, sleep_queue_out) = unbounded();

        Self {
            inactive_actors: Arc::new(DashMap::new()),
            wakeup_queue_in,
            wakeup_queue_out,
            sleep_queue_in,
            sleep_queue_out
        }
    }

    pub fn add_sleeping_actor(&self, address: ActorAddress, actor: Arc<RwLock<dyn ExecutorTrait>>, sleep: Duration) {
        self.add_inactive_actor(address.clone(), actor);
        self.sleep_queue_in.send(Sleep {
            now: Instant::now(),
            duration: sleep,
            actor_address: address
        }).unwrap();
    }

    pub fn add_inactive_actor(&self, address: ActorAddress, actor: Arc<RwLock<dyn ExecutorTrait>>) {
        self.inactive_actors.insert(address, actor);
    }

    pub fn wakeup(&self, address: ActorAddress) {
        self.wakeup_queue_in
            .send(Wakeup {
                actor_address: address,
                iteration: 0,
            })
            .unwrap();
    }

    pub fn manage_sleeping(&self, system_status: SystemState) {
        let recv_timeout = Duration::from_millis(1000);
        loop {
            if system_status.is_stopped() || system_status.is_stopping() {
                return;
            }

            let msg = self.sleep_queue_out.recv_timeout(recv_timeout);
            if msg.is_err() {
                continue;
            }
            let sleep_msg = msg.unwrap();
            let duration = sleep_msg.now.elapsed();
            if duration >= sleep_msg.duration {
                self.wakeup_queue_in.send(Wakeup {
                    iteration: 1,
                    actor_address: sleep_msg.actor_address
                }).unwrap();
                continue
            }

            sleep(Duration::from_millis(1000));
            self.sleep_queue_in.send(sleep_msg).unwrap();
            continue;

        }
    }

    pub fn manage_inactive(&self, system_status: SystemState, thread_pool_manager: ThreadPoolManager) {
        let mut wake_deduplication: HashMap<ActorAddress, Instant> = HashMap::new();
        let recv_timeout = Duration::from_secs(1);
        loop {
            if system_status.is_stopped() {
                return;
            }
            if system_status.is_stopping() {
                let mut keys: Vec<ActorAddress> = Vec::new();
                for key in self.inactive_actors.iter() {
                    keys.push(key.key().clone());
                }
                for key in keys {
                    let inactive_actor = self.inactive_actors.remove(&key).unwrap();
                    let pool_name = inactive_actor.0.pool;
                    let actor_ref = inactive_actor.1;
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
                if duration >= Duration::from_millis(4000) {
                    wake_deduplication.remove(&wakeup_message.actor_address);
                } else {
                    continue;
                }
            }

            wake_deduplication.insert(wakeup_message.actor_address.clone(), Instant::now());
            if !self
                .inactive_actors
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
                .inactive_actors
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
