use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ActorSystemStatus {
    total_actor_count: Arc<AtomicUsize>,
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    is_force_stopped: Arc<AtomicBool>,
}

impl ActorSystemStatus {
    pub fn new() -> Self {
        Self {
            total_actor_count: Arc::new(AtomicUsize::new(0)),
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            is_force_stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&self) {
        self.is_stopping.store(true, Ordering::Relaxed);
    }

    pub fn finalize_stop(&self, force: bool) {
        if force {
            self.is_force_stopped.store(true, Ordering::Relaxed);
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

    pub fn increment_actor_count(&self) {
        self.total_actor_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_actor_count(&self) {
        self.total_actor_count.fetch_sub(1, Ordering::Relaxed);

    }

    pub fn get_actor_count(&self) -> usize {
        self.total_actor_count.load(Ordering::Relaxed)
    }
}