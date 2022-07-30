use crate::actor::actor_address::ActorAddress;
use crate::message::serialized_message::SerializedMessage;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::actor::mailbox::{Mailbox, BaseMailbox};
use crate::prelude::{ActorWrapper, Handler};
use crate::system::wakeup_manager::WakeupManager;

#[derive(Clone)]
pub struct SystemState {
    mailboxes: Arc<DashMap<ActorAddress, Arc<dyn BaseMailbox>>>,
    wakeup_manager: WakeupManager,
    total_actor_count: Arc<AtomicUsize>,
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    is_force_stopped: Arc<AtomicBool>,
}

impl SystemState {
    pub fn new(wakeup_manager: WakeupManager) -> Self {
        Self {
            mailboxes: Arc::new(DashMap::new()),
            wakeup_manager,
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
                self.mailboxes.clear();
                break;
            }
            sleep(Duration::from_millis(1));
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
        let target = self.mailboxes.get(address);
        if target.is_some() {
            let target = target.unwrap();
            target.send_serialized(msg);
        }
    }

    pub fn remove_mailbox(&self, address: &ActorAddress) {
        self.total_actor_count.fetch_sub(1, Ordering::Relaxed);
        self.mailboxes.remove(address);
    }


    pub fn add_mailbox<A>(&self, address: ActorAddress, mailbox: Mailbox<A>)
    where
        A: Handler<SerializedMessage> + 'static
    {
        self.total_actor_count.fetch_add(1, Ordering::Relaxed);
        self.mailboxes.insert(address, Arc::new(mailbox));
    }

    pub fn get_actor_ref<A>(&self, address: ActorAddress) -> Option<ActorWrapper<A>>
        where
            A: Handler<SerializedMessage> + 'static
    {
        let mb = self.mailboxes.get(&address).unwrap().value().clone();
        return match mb.as_any().downcast_ref::<Mailbox<A>>() {
            Some(m) => Some(ActorWrapper::new(m.clone(), address, self.wakeup_manager.clone())),
            None => None
        };
    }

    pub fn is_mailbox_active(&self, address: &ActorAddress) -> bool {
        self.mailboxes.contains_key(address)
    }
}
