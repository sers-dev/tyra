use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_config::ActorConfig;
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::executor::{Executor, ExecutorTrait};
use crate::actor::mailbox::Mailbox;
use crate::config::tyra_config::DEFAULT_POOL;
use crate::prelude::{Actor, Handler, SerializedMessage};
use crate::system::actor_error::ActorError;
use crate::system::actor_system::ActorSystem;
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::system_state::SystemState;
use crate::system::wakeup_manager::WakeupManager;
use dashmap::DashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

/// Used to create [Actor]s in the [ActorSystem]
///
/// Each builder has access to all `Mailbox<A>` objects within the `ActorSystem` and is able to provide a copy to an existing `ActorRef<A>` if the address is already in use
///
/// See [.spawn()](#method.spawn) for a detailed explanation
#[derive(Clone)]
pub struct ActorBuilder<A>
where
    A: Actor + 'static,
{
    existing: Arc<DashMap<ActorAddress, ActorWrapper<A>>>,
    system: ActorSystem,
    system_state: SystemState,
    wakeup_manager: WakeupManager,
    internal_actor_manager: InternalActorManager,
    actor_config: ActorConfig,
}

impl<A> ActorBuilder<A>
where
    A: Actor + Handler<SerializedMessage> + 'static,
{
    /// This is called through [ActorSystem.builder](../prelude/struct.ActorSystem.html#method.builder)
    pub fn new(
        system: ActorSystem,
        system_state: SystemState,
        wakeup_manager: WakeupManager,
        internal_actor_manager: InternalActorManager,
    ) -> ActorBuilder<A> {
        let config = system.get_config();

        let actor_config = ActorConfig {
            pool_name: String::from(DEFAULT_POOL),
            mailbox_size: config.general.default_mailbox_size,
            message_throughput: config.general.default_message_throughput,
        };

        ActorBuilder {
            existing: Arc::new(DashMap::new()),
            system,
            system_state,
            wakeup_manager,
            internal_actor_manager,
            actor_config,
        }
    }

    pub fn set_pool_name(mut self, pool_name: impl Into<String>) -> ActorBuilder<A> {
        self.actor_config.pool_name = pool_name.into();
        self
    }

    pub fn set_message_throughput(mut self, message_throughput: usize) -> ActorBuilder<A> {
        self.actor_config.message_throughput = message_throughput;
        self
    }

    pub fn set_mailbox_unbounded(self) -> ActorBuilder<A> {
        self.set_mailbox_size(0)
    }

    pub fn set_mailbox_size(mut self, mailbox_size: usize) -> ActorBuilder<A> {
        self.actor_config.mailbox_size = mailbox_size;
        self
    }

    /// Creates the defined [Actor] on the [ActorSystem]
    ///
    /// # Returns
    ///
    /// `Ok(ActorWrapper<A>)` if actor was created successfully
    ///
    /// `Ok(ActorWrapper<A>)` if the actor is already running on the system
    ///
    /// `Err(ActorError)` see [ActorError](../prelude/enum.ActorError.html) for detailed information
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {}
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// struct BrokenActor {}
    /// impl BrokenActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for BrokenActor {}
    ///
    /// struct BrokenActorFactory {}
    /// impl BrokenActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<BrokenActor> for BrokenActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<BrokenActor>) -> Result<BrokenActor, Box<dyn Error>> {
    ///         let error = std::io::Error::from_raw_os_error(1337);
    ///         return Err(Box::new(error));
    ///     }
    /// }
    ///
    /// #[ntest::timeout(100000)]
    /// fn main() {
    ///     let mut actor_config = TyraConfig::new().unwrap();
    ///     actor_config.thread_pool.config.insert(String::from("default"), ThreadPoolConfig::new(1, 1, 1, 1.0));
    ///     let actor_system = ActorSystem::new(actor_config);
    ///
    ///     //this does not work, because although there's not yet an actor called `broken` on the pool the `new_actor` method returns an error
    ///     let this_is_not_working = actor_system.builder().spawn("broken", BrokenActorFactory::new());
    ///     assert!(this_is_not_working.is_err(), "The BrokenActor was spawned");
    ///     let err = this_is_not_working.err().unwrap();
    ///     assert_eq!(err, ActorError::InitError, "Error is not correct");
    ///
    ///     let actor_name = "test";
    ///     //this works, because there's no actor called `test` yet on the pool
    ///     let this_works = actor_system.builder().spawn(actor_name, TestActorFactory::new());
    ///     assert!(this_works.is_ok(), "The actor could not be spawned");
    ///
    ///     //this works, because there's already an actor called `test` with type `TestActor` on the pool, therefore the result is the same actor that was created in the previous spawn command
    ///     let this_works_as_well = actor_system.builder().spawn(actor_name, TestActorFactory::new());
    ///     assert!(this_works_as_well.is_ok(), "The `ActorWrapper` could not be fetched");
    ///
    ///     //this does not work, because the pool is currently configured to only allow a single actor
    ///     let pool_full = actor_system.builder().spawn("full", TestActorFactory::new());
    ///     assert!(pool_full.is_err(), "The actor could not be spawned");
    ///     let err = pool_full.err().unwrap();
    ///     assert_eq!(err, ActorError::ThreadPoolHasTooManyActorsError, "Error is not correct");
    ///
    ///     ////this does not work, because the pool does not exist in the configuration
    ///     let invalid_pool = actor_system.builder().set_pool_name("invalid").spawn(actor_name, TestActorFactory::new());
    ///     assert!(invalid_pool.is_err(), "The Actor was spawned");
    ///     let err = invalid_pool.err().unwrap();
    ///     assert_eq!(err, ActorError::ThreadPoolDoesNotExistError, "Error is not correct");
    ///
    ///     ////this does not work, because there's already an actor called `test` with a different type on the pool
    ///     let this_is_not_working_either = actor_system.builder().spawn(actor_name, BrokenActorFactory::new());
    ///     assert!(this_is_not_working_either.is_err(), "Illegal Actor type conversion");
    ///     let err = this_is_not_working_either.err().unwrap();
    ///     assert_eq!(err, ActorError::InvalidActorTypeError, "Error is not correct");
    ///
    ///     actor_system.stop(Duration::from_millis(3000));
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    pub fn spawn<P>(&self, name: impl Into<String>, props: P) -> Result<ActorWrapper<A>, ActorError>
    where
        P: ActorFactory<A> + 'static,
    {
        let actor_address = ActorAddress {
            actor: name.into(),
            system: String::from(self.system.get_name()),
            pool: self.actor_config.pool_name.clone(),
            remote: String::from("local"),
        };

        if self.system_state.is_mailbox_active(&actor_address) {
            return self
                .system_state
                .get_actor_ref(actor_address, self.internal_actor_manager.clone());
        }

        let result = self.system_state.increase_pool_actor_count(&actor_address);
        if result.is_err() {
            return Err(result.unwrap_err());
        }

        let (sender, receiver) = if self.actor_config.mailbox_size == 0 {
            flume::unbounded()
        } else {
            flume::bounded(self.actor_config.mailbox_size)
        };

        let mailbox = Mailbox {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            msg_in: sender,
        };

        let actor_ref = ActorWrapper::new(
            mailbox.clone(),
            actor_address.clone(),
            self.wakeup_manager.clone(),
            self.internal_actor_manager.clone(),
        );

        let actor_handler = Executor::new(
            props,
            actor_address.clone(),
            self.actor_config.clone(),
            mailbox.clone(),
            receiver,
            self.system.clone(),
            actor_ref.clone(),
        );

        match actor_handler {
            Ok(a) => {
                self.system_state.add_mailbox(actor_address.clone(), mailbox);
                self.wakeup_manager.add_inactive_actor(a.get_address(), Arc::new(RwLock::new(a)));
                self.existing.insert(actor_address, actor_ref.clone());
                return Ok(actor_ref);
            }
            Err(e) => {
                self.system_state.decrease_pool_actor_count(&actor_address);
                return Err(e);
            }
        }
    }

    pub fn spawn_multiple<P>(&self, name: impl Into<String> + Clone + std::fmt::Display, props: P, spawn_count: usize) -> Result<Vec<ActorWrapper<A>>, ActorError>
        where
            P: ActorFactory<A> + 'static + Clone,
    {
        let mut to_return = Vec::new();
        for i in 0..spawn_count {
            let name = format!("{}-{}", name.clone(), i);
            let res = self.spawn(name, props.clone());
            match res {
                Ok(res) => {
                    to_return.push(res);
                }
                Err(e) => {
                    for actor in &to_return {
                        let _ = actor.stop();
                    }
                    return Err(e);
                }
            }
        }

        return Ok(to_return);
    }


}
