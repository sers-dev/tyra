use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_config::ActorConfig;
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::executor::{Executor, ExecutorTrait};
use crate::actor::mailbox::Mailbox;
use crate::config::tyra_config::DEFAULT_POOL;
use crate::prelude::{Actor, Handler, SerializedMessage};
use crate::system::actor_system::ActorSystem;
use crate::system::system_state::SystemState;
use crate::system::wakeup_manager::WakeupManager;
use crossbeam_channel::{bounded, unbounded};
use dashmap::DashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use crate::system::actor_error::ActorError;
use crate::system::internal_actor_manager::InternalActorManager;

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
    /// `Err(ActorError)` see
    ///
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
            return self.system_state.get_actor_ref(actor_address, self.internal_actor_manager.clone());
        }

        let (sender, receiver) = if self.actor_config.mailbox_size == 0 {
            unbounded()
        } else {
            bounded(self.actor_config.mailbox_size)
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
                let result = self.system_state
                    .add_mailbox(actor_address.clone(), mailbox);

                if result.is_err() {
                    return Err(result.unwrap_err());
                }

                self.wakeup_manager.add_inactive_actor(
                    a.get_address(),
                    Arc::new(RwLock::new(a)),
                );

                self.existing.insert(actor_address, actor_ref.clone());
                return Ok(actor_ref);
            },
            Err(e) => {
                return Err(e);
            }
        }

    }
}
