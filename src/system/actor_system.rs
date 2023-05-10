use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_builder::ActorBuilder;
use crate::config::pool_config::ThreadPoolConfig;
use crate::config::tyra_config::{TyraConfig, DEFAULT_POOL};
use crate::message::serialized_message::SerializedMessage;
use crate::prelude::{Actor, ActorError, Handler};
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::system_state::SystemState;
use crate::system::thread_pool_manager::ThreadPoolManager;
use crate::system::wakeup_manager::WakeupManager;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// Manages thread pools and actors
#[derive(Clone)]
pub struct ActorSystem {
    state: SystemState,
    thread_pool_manager: ThreadPoolManager,
    wakeup_manager: WakeupManager,
    name: String,
    hostname: String,
    config: Arc<TyraConfig>,
    internal_actor_manager: InternalActorManager,
    sigint_received: Arc<AtomicBool>,
}

impl ActorSystem {
    /// Creates and starts a new ActorSystem based on supplied configuration
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem};
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// ```
    pub fn new(config: TyraConfig) -> Self {
        if config.general.override_panic_hook {
            std::panic::set_hook(Box::new(|_| {}));
        }

        let thread_pool_config = config.thread_pool.clone();

        let thread_pool_manager = ThreadPoolManager::new();
        let wakeup_manager = WakeupManager::new();

        let thread_pool_max_actors = DashMap::new();

        for (key, value) in thread_pool_config.config.iter() {
            thread_pool_manager.add_pool_with_config(key, value.clone());
            thread_pool_max_actors.insert(key.clone(), value.actor_limit);
        }

        //start the net_manager and provide the net_worker_lb_address to the systemstate
        //also properly fill out remote_name in ActorAddress. hostname of the system is probably a good choice, will in most cases be equal to the system_name but that's okay
        let net_worker_lb_address = ActorAddress::new(config.general.hostname.clone(), config.general.name.clone(), "", "");
        let state = SystemState::new(wakeup_manager.clone(), Arc::new(thread_pool_max_actors), net_worker_lb_address, config.general.name.clone(), config.general.hostname.clone());

        let s = state.clone();
        let t = thread_pool_manager.clone();
        let w = wakeup_manager.clone();
        std::thread::spawn(move || t.manage(s, w));
        let s = state.clone();
        let t = thread_pool_manager.clone();
        let w = wakeup_manager.clone();
        std::thread::spawn(move || w.manage_inactive(s, t));
        let w = wakeup_manager.clone();
        let s = state.clone();
        std::thread::spawn(move || w.clone().manage_sleeping(s));

        let mut system = ActorSystem {
            state,
            thread_pool_manager,
            wakeup_manager,
            name: config.general.name.clone(),
            hostname: config.general.hostname.clone(),
            config: Arc::new(config.clone()),
            internal_actor_manager: InternalActorManager::new(),
            sigint_received: Arc::new(AtomicBool::new(false)),
        };

        if config.general.signal_graceful_timeout_in_seconds > 0 {
            let sys = system.clone();
            ctrlc::set_handler(move || {
                sys.sigint_handler(Duration::from_secs(300));
            })
            .unwrap();
        }

        system.internal_actor_manager.init(system.clone());

        system
    }

    /// Adds a new named pool using the [default pool configuration](https://github.com/sers-dev/tyra/blob/master/src/config/default.toml)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use tyra::prelude::{TyraConfig, ActorSystem};
    ///
    /// let mut actor_config = TyraConfig::new().unwrap();
    /// //disable automatic setup of sigint handling, so that we can set it manually
    /// actor_config.general.signal_graceful_timeout_in_seconds = 0;
    /// let actor_system = ActorSystem::new(actor_config);
    /// ctrlc::set_handler(move || {actor_system.sigint_handler(Duration::from_secs(60));}).unwrap();
    /// ```
    pub fn sigint_handler(&self, graceful_termination_timeout: Duration) {
        if self.sigint_received.load(Ordering::Relaxed) {
            self.force_stop();
        }
        self.sigint_received.store(true, Ordering::Relaxed);
        self.stop(graceful_termination_timeout);
    }
    /// Adds a new named pool using the [default pool configuration](https://github.com/sers-dev/tyra/blob/master/src/config/default.toml)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem};
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// actor_system.add_pool("test");
    /// ```
    pub fn add_pool(&self, name: &str) {
        let default_config = self.config.thread_pool.config.get(DEFAULT_POOL).unwrap();
        let config = self
            .config
            .thread_pool
            .config
            .get(name)
            .unwrap_or(default_config);
        self.add_pool_with_config(name, config.clone());
    }

    /// Adds a new named pool with custom [pool configuration](../prelude/struct.ThreadPoolConfig.html)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let pool_config = ThreadPoolConfig::new(0, 2, 4, 1.0);
    /// actor_system.add_pool_with_config("test", pool_config);
    /// ```
    pub fn add_pool_with_config(&self, name: &str, thread_pool_config: ThreadPoolConfig) {
        self.state
            .add_pool_actor_limit(String::from(name.clone()), thread_pool_config.actor_limit);
        self.thread_pool_manager
            .add_pool_with_config(name, thread_pool_config);
    }

    /// Returns the amount of Actors that can still be put onto a given thread_pool
    ///
    /// If the thread_pool does not have a configured limit, returns (usize::Max - current_actor_count)
    ///
    /// Returns ActorError::ThreadPoolDoesNotExistError if the given `pool_name` does not exist
    pub fn get_available_actor_count_for_pool(&self, pool_name: &str) -> Result<usize, ActorError> {
        return self.state.get_available_actor_count_for_pool(pool_name);
    }

    /// Sends a [SerializedMessage](../prelude/struct.SerializedMessage.html) to an Actor by Address
    ///
    /// # Important Note
    ///
    /// This function send a SerializedMessage to the Actor, which is implemented to call the [ActorTrait.handle_serialized_message](../prelude/trait.Actor.html#method.handle_serialized_message)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::error::Error;
    /// use serde::Serialize;
    /// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor, ActorResult, ActorMessage};
    ///
    /// struct TestActor {}
    ///
    /// #[derive(Hash, Serialize)]
    /// struct HelloWorld {}
    /// impl ActorMessage for HelloWorld {}
    /// impl Actor for TestActor {
    ///     fn handle_serialized_message(&mut self, _msg: SerializedMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.actor_ref.send(HelloWorld{});
    ///         Ok(ActorResult::Ok)
    ///     }
    ///
    /// }
    ///
    /// impl Handler<HelloWorld> for TestActor {
    ///     fn handle(&mut self, _msg: HelloWorld, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         Ok(ActorResult::Ok)
    ///     }
    /// }
    ///
    /// struct TestFactory {}
    ///
    /// impl ActorFactory<TestActor> for TestFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor {})
    ///     }
    /// }
    ///
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let actor_wrapper = actor_system.builder().spawn("test", TestFactory{}).unwrap();
    /// let address = actor_wrapper.get_address();
    /// actor_system.send_to_address(address, Vec::new());
    /// ```
    pub fn send_to_address<>(&self, address: &ActorAddress, msg: Vec<u8>) {
        self.state.send_to_address(address, msg);
    }

    /// Returns a Builder to configure and spawn an actor in the system
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use std::error::Error;
    /// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor};
    ///
    /// struct TestActor {}
    ///
    /// impl Actor for TestActor {}
    ///
    /// struct TestFactory {}
    ///
    /// impl ActorFactory<TestActor> for TestFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor {})
    ///     }
    /// }
    ///
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let builder = actor_system.builder();
    /// builder.spawn("test", TestFactory{}).unwrap();
    /// ```
    pub fn builder<A>(&self) -> ActorBuilder<A>
    where
        A: Handler<SerializedMessage> + Actor,
    {
        ActorBuilder::new(
            self.clone(),
            self.state.clone(),
            self.wakeup_manager.clone(),
            self.internal_actor_manager.clone(),
        )
    }

    /// Sends a SystemStopMessage to all running Actors, and wakes them up if necessary.
    /// Users can implement their own clean system stop behavior, by implementing [Actor.on_system_stop](../prelude/trait.Actor.html#method.on_system_stop) and [Actor.on_actor_stop](../prelude/trait.Actor.html#method.on_actor_stop)
    ///
    /// System will stop after all actors have been stopped or after `graceful_termination_timeout`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// actor_system.stop(Duration::from_secs(1));
    /// ```
    pub fn stop(&self, graceful_termination_timeout: Duration) {
        self.state.stop(graceful_termination_timeout);
    }

    pub fn force_stop(&self) {
        self.state.force_stop();
    }

    /// Same as stop, but with fixed user defined exit code
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// actor_system.stop_with_code(Duration::from_secs(1), -1);
    /// ```
    pub fn stop_with_code(&self, graceful_termination_timeout: Duration, code: i32) {
        self.state.use_forced_exit_code(code);
        self.stop(graceful_termination_timeout);
    }

    /// Waits for the system to stop
    ///
    /// # Returns
    ///
    /// `0 as i32` if cleanly stopped by removing all actors from system
    ///
    /// `1 as i32` if force stopped after stop timeout
    ///
    /// `i32` if stop was called through `stop_with_code`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    /// use std::process::exit;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// actor_system.stop(Duration::from_secs(3));
    /// exit(actor_system.await_shutdown());
    /// ```
    pub fn await_shutdown(&self) -> i32 {
        while !self.state.is_stopped() {
            sleep(Duration::from_millis(500));
        }
        return self.state.get_exit_code();
    }

    /// Returns a reference to the [TyraConfig](../prelude/struct.TyraConfig.html)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    /// use std::process::exit;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let conf = actor_system.get_config();
    /// ```
    pub fn get_config(&self) -> &TyraConfig {
        &self.config
    }

    /// Returns the configured name of the system
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    /// use std::process::exit;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let name = actor_system.get_name();
    /// ```
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the configured hostname of the system
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ThreadPoolConfig};
    /// use std::time::Duration;
    /// use std::process::exit;
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let name = actor_system.get_hostname();
    /// ```
    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }
}
