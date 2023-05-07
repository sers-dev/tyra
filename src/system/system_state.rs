use crate::actor::actor_address::ActorAddress;
use crate::actor::mailbox::{BaseMailbox, Mailbox};
use crate::message::serialized_message::SerializedMessage;
use crate::prelude::{ActorWrapper, Handler};
use crate::system::actor_error::ActorError;
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::wakeup_manager::WakeupManager;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct SystemState {
    mailboxes: Arc<DashMap<ActorAddress, Arc<dyn BaseMailbox>>>,
    wakeup_manager: WakeupManager,
    total_actor_count: Arc<AtomicUsize>,
    pool_actor_count: Arc<DashMap<String, AtomicUsize>>,
    max_actors_per_pool: Arc<DashMap<String, usize>>,
    is_stopped: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    is_force_stopped: Arc<AtomicBool>,
    forced_exit_code: Arc<AtomicI32>,
    use_forced_exit_code: Arc<AtomicBool>,
}

impl SystemState {
    pub fn new(
        wakeup_manager: WakeupManager,
        max_actors_per_pool: Arc<DashMap<String, usize>>,
    ) -> Self {
        Self {
            mailboxes: Arc::new(DashMap::new()),
            wakeup_manager,
            total_actor_count: Arc::new(AtomicUsize::new(0)),
            pool_actor_count: Arc::new(DashMap::new()),
            max_actors_per_pool,
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            is_force_stopped: Arc::new(AtomicBool::new(false)),
            forced_exit_code: Arc::new(AtomicI32::new(0)),
            use_forced_exit_code: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn force_stop(&self) {
        self.is_force_stopped.store(true, Ordering::Relaxed);
        self.stop(Duration::from_secs(1));
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
            if now.elapsed() >= timeout || self.is_force_stopped.load(Ordering::Relaxed) {
                self.is_force_stopped.store(true, Ordering::Relaxed);
                self.mailboxes.clear();
                break;
            }
            sleep(Duration::from_millis(100));
        }
        self.is_stopped.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }

    pub fn is_stopping(&self) -> bool {
        self.is_stopping.load(Ordering::Relaxed)
    }

    pub fn use_forced_exit_code(&self, code: i32) {
        self.forced_exit_code.store(code, Ordering::Relaxed);
        self.use_forced_exit_code.store(true, Ordering::Relaxed);
    }

    pub fn get_exit_code(&self) -> i32 {
        if self.use_forced_exit_code.load(Ordering::Relaxed) {
            return self.forced_exit_code.load(Ordering::Relaxed);
        }
        return self.is_force_stopped() as i32;
    }

    fn is_force_stopped(&self) -> bool {
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
            if target.is_sleeping() {
                self.wakeup_manager.wakeup(target.key().clone());
            }
        }
    }

    pub fn increase_pool_actor_count(&self, address: &ActorAddress) -> Result<(), ActorError> {
        let maximum_actor_count = self.max_actors_per_pool.get(&address.pool);
        if maximum_actor_count.is_none() {
            return Err(ActorError::ThreadPoolDoesNotExistError);
        }
        let maximum_actor_count = maximum_actor_count.unwrap();
        let maximum_actor_count = *maximum_actor_count.value();

        let current_pool_count = self
            .pool_actor_count
            .entry(address.pool.clone())
            .or_insert(AtomicUsize::new(0));
        let current_pool_count = current_pool_count.value();

        let current = current_pool_count.load(Ordering::Relaxed);
        if maximum_actor_count != 0 as usize && maximum_actor_count <= current {
            return Err(ActorError::ThreadPoolHasTooManyActorsError);
        }

        current_pool_count.fetch_add(1, Ordering::Relaxed);
        self.total_actor_count.fetch_add(1, Ordering::Relaxed);

        return Ok(());
    }

    pub fn decrease_pool_actor_count(&self, address: &ActorAddress) {
        self.pool_actor_count
            .entry(address.pool.clone())
            .and_modify(|v| {
                v.fetch_sub(1, Ordering::Relaxed);
            });
        self.total_actor_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn remove_mailbox(&self, address: &ActorAddress) {
        self.decrease_pool_actor_count(address);
        self.mailboxes.remove(address);
    }

    pub fn add_mailbox<A>(&self, address: ActorAddress, mailbox: Mailbox<A>)
    where
        A: Handler<SerializedMessage> + 'static,
    {
        self.mailboxes.insert(address, Arc::new(mailbox));
    }

    pub fn add_pool_actor_limit(&self, pool_name: String, max_actors: usize) {
        self.max_actors_per_pool.insert(pool_name, max_actors);
    }

    pub fn get_available_actor_count_for_pool(&self, pool_name: &str) -> Result<usize, ActorError> {
        let maximum_actor_count = self.max_actors_per_pool.get(pool_name);
        if maximum_actor_count.is_none() {
            return Err(ActorError::ThreadPoolDoesNotExistError);
        }

        let maximum_actor_count = maximum_actor_count.unwrap();
        let maximum_actor_count = *maximum_actor_count.value();

        let current_pool_count = self.pool_actor_count.get(pool_name);

        let current_pool_count = if current_pool_count.is_some() {
            let current_pool_count = current_pool_count.unwrap();
            current_pool_count.value().load(Ordering::Relaxed)
        } else {
            0 as usize
        };

        if maximum_actor_count == 0 {
            let result = usize::MAX - current_pool_count;
            return Ok(result);
        }

        let result = maximum_actor_count - current_pool_count;
        return Ok(result);
    }

    pub fn get_actor_ref<A>(
        &self,
        address: &ActorAddress,
        internal_actor_manager: InternalActorManager,
    ) -> Result<ActorWrapper<A>, ActorError>
    where
        A: Handler<SerializedMessage> + 'static,
    {
        let mb = self.mailboxes.get(address).unwrap().value().clone();
        return match mb.as_any().downcast_ref::<Mailbox<A>>() {
            Some(m) => Ok(ActorWrapper::new(
                m.clone(),
                address.clone(),
                self.wakeup_manager.clone(),
                internal_actor_manager,
                self.clone(),
            )),
            None => Err(ActorError::InvalidActorTypeError),
        };
    }

    pub fn is_mailbox_active(&self, address: &ActorAddress) -> bool {
        self.mailboxes.contains_key(address)
    }
}
