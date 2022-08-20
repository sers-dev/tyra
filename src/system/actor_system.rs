use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_builder::ActorBuilder;
use crate::config::pool_config::ThreadPoolConfig;
use crate::config::tyra_config::{TyraConfig, DEFAULT_POOL};
use crate::message::serialized_message::SerializedMessage;
use crate::prelude::{Actor, Handler};
use crate::system::system_state::SystemState;
use crate::system::thread_pool_manager::ThreadPoolManager;
use crate::system::wakeup_manager::WakeupManager;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use crate::system::internal_actor_manager::InternalActorManager;

/// Manages thread pools and actors
#[derive(Clone)]
pub struct ActorSystem {
    state: SystemState,
    thread_pool_manager: ThreadPoolManager,
    wakeup_manager: WakeupManager,
    name: String,
    config: Arc<TyraConfig>,
    internal_actor_manager: InternalActorManager
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
        let state = SystemState::new(wakeup_manager.clone());

        for (key, value) in thread_pool_config.config.iter() {
            thread_pool_manager.add_pool_with_config(key, value.clone());
        }

        let s = state.clone();
        let t = thread_pool_manager.clone();
        let w = wakeup_manager.clone();
        std::thread::spawn(move || t.manage(s, w));
        let s = state.clone();
        let t = thread_pool_manager.clone();
        let w = wakeup_manager.clone();
        std::thread::spawn(move || w.manage(s, t));

        let mut system = ActorSystem {
            state,
            thread_pool_manager,
            wakeup_manager,
            name: config.general.name.clone(),
            config: Arc::new(config.clone()),
            internal_actor_manager: InternalActorManager::new(),
        };

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
        self.thread_pool_manager
            .add_pool_with_config(name, thread_pool_config);
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
    /// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor, ActorMessage, ActorResult};
    ///
    /// struct TestActor {}
    ///
    /// struct HelloWorld {}
    /// impl ActorMessage for HelloWorld {}
    /// impl Actor for TestActor {
    ///     fn handle_serialized_message(&mut self, _msg: SerializedMessage, context: &ActorContext<Self>) -> ActorResult {
    ///         context.actor_ref.send(HelloWorld{});
    ///         ActorResult::Ok
    ///     }
    ///
    /// }
    ///
    /// impl Handler<HelloWorld> for TestActor {
    ///     fn handle(&mut self, _msg: HelloWorld, _context: &ActorContext<Self>) -> ActorResult {
    ///         ActorResult::Ok
    ///     }
    /// }
    ///
    /// struct TestFactory {}
    ///
    /// impl ActorFactory<TestActor> for TestFactory {
    ///     fn new_actor(&self, _context: ActorContext<TestActor>) -> TestActor {
    ///         TestActor {}
    ///     }
    /// }
    ///
    ///
    /// let actor_config = TyraConfig::new().unwrap();
    /// let actor_system = ActorSystem::new(actor_config);
    /// let actor_wrapper = actor_system.builder().spawn("test", TestFactory{}).unwrap();
    /// let address = actor_wrapper.get_address();
    /// actor_system.send_to_address(address, SerializedMessage::new(Vec::new()));
    /// ```
    pub fn send_to_address(&self, address: &ActorAddress, msg: SerializedMessage) {
        self.state.send_to_address(address, msg);
    }

    /// Returns a Builder to configure and spawn an actor in the system
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor};
    ///
    /// struct TestActor {}
    ///
    /// impl Actor for TestActor {}
    ///
    /// struct TestFactory {}
    ///
    /// impl ActorFactory<TestActor> for TestFactory {
    ///     fn new_actor(&self, _context: ActorContext<TestActor>) -> TestActor {
    ///         TestActor {}
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

    /// Waits for the system to stop
    ///
    /// # Returns
    ///
    /// `0 as i32` if cleanly stopped by removing all actors from system
    ///
    /// `1 as i32` if force stopped after stop timeout
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
    /// actor_system.stop(Duration::from_secs(1));
    /// exit(actor_system.await_shutdown());
    /// ```
    pub fn await_shutdown(&self) -> i32 {
        while !self.state.is_stopped() {
            sleep(Duration::from_millis(1));
        }
        self.state.is_force_stopped() as i32
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

}
