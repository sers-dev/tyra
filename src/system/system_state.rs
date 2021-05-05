use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use dashmap::DashMap;
use crate::actor::actor_address::ActorAddress;
use crate::actor::actor::Actor;
use crate::message::serialized_message::SerializedMessage;
use std::thread::sleep;
use std::time::{Instant, Duration};

#[derive(Clone)]
pub struct SystemState {
    actors: DashMap<ActorAddress, Arc<dyn Actor>>,
    total_actor_count: Arc<AtomicUsize>,
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    is_force_stopped: Arc<AtomicBool>,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            actors: DashMap::new(),
            total_actor_count: Arc::new(AtomicUsize::new(0)),
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            is_force_stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&self, graceful_termination_timeout: Duration) {
        if self.is_stopping() {
            return;
        }
        self.is_stopping.store(true, Ordering::Relaxed);
        let s = self.clone();
        std::thread::spawn(move || s.shutdown(graceful_termination_timeout));
    }

    fn shutdown(&self, timeout: Duration) {
        let now = Instant::now();
        while self.get_actor_count() != 0 {
            if now.elapsed() >= timeout {
                self.is_force_stopped.store(true, Ordering::Relaxed);
                self.actors.clear();
                break;
            }
            sleep(timeout / 10);
        }
        self.is_stopped.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }

    pub fn is_stopping(&self) -> bool {
        self.is_stopping.load(Ordering::Relaxed)
    }

    pub fn is_force_stopped(&self) -> bool {
        self.is_force_stopped.load(Ordering::Relaxed)
    }

    pub fn get_actor_count(&self) -> usize {
        self.total_actor_count.load(Ordering::Relaxed)
    }

    pub fn send_to_address(&self, address: &ActorAddress, msg: SerializedMessage) {
        let target = self.actors.get(address);
        if target.is_some() {
            let target = target.unwrap();
            target.handle_serialized_message(msg);
        }
    }

    pub fn remove_actor(&self, address: &ActorAddress) {
        self.total_actor_count.fetch_sub(1, Ordering::Relaxed);
        self.actors.remove(address);
    }

    pub fn add_actor(&self, address: ActorAddress, actor: Arc<dyn Actor>) {
        self.total_actor_count.fetch_add(1, Ordering::Relaxed);
        self.actors.insert(address, actor);
    }


}