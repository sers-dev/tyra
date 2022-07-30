use crate::actor::actor::Actor;
use crate::actor::actor_config::{ActorConfig, RestartPolicy};
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::config::tyra_config::DEFAULT_POOL;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;
use crossbeam_channel::{unbounded, bounded};
use crate::actor::mailbox::Mailbox;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use crate::actor::actor_address::ActorAddress;
use crate::actor::executor::{Executor, ExecutorTrait};
use crate::system::wakeup_manager::WakeupManager;
use crate::system::system_state::SystemState;
use dashmap::DashMap;
use crate::prelude::{ActorMessageDeserializer, Handler, SerializedMessage};

/// Used to create [Actor]s in the [ActorSystem]
///
/// Each builder keeps a clone-safe storage of already created Actors.
///
/// In case the same `ActorAddress` is used multiple times with the same builder for an already running actor, it will simply return the `ActorWrapper<A>` without creating the actor a second time.
/// See [.spawn()](#method.spawn) for a detailed explanation
#[derive(Clone)]
pub struct ActorBuilder<A>
where
    A: Actor + UnwindSafe + 'static + ActorMessageDeserializer,
{
    existing: Arc<DashMap<ActorAddress, ActorWrapper<A>>>,
    system: ActorSystem,
    system_state: SystemState,
    wakeup_manager: WakeupManager,
    actor_config: ActorConfig,
}

impl<A> ActorBuilder<A>
    where
        A: Actor + UnwindSafe + 'static + Handler<SerializedMessage> + ActorMessageDeserializer,
{
    /// This is called through [ActorSystem.builder](../prelude/struct.ActorSystem.html#method.builder)
    pub fn new(system: ActorSystem, system_state: SystemState, wakeup_manager: WakeupManager) -> ActorBuilder<A> {
        let config = system.get_config();

        let actor_config = ActorConfig {
            pool_name: String::from(DEFAULT_POOL),
            mailbox_size: config.general.default_mailbox_size,
            message_throughput: config.general.default_message_throughput,
            restart_policy: config.general.default_restart_policy,
        };

        ActorBuilder {
            existing: Arc::new(DashMap::new()),
            system,
            system_state,
            wakeup_manager,
            actor_config,
        }
    }

    pub fn set_restart_policy(mut self, restart_policy: RestartPolicy) -> ActorBuilder<A> {
        self.actor_config.restart_policy = restart_policy;
        self
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
    /// `Some(ActorWrapper<A>)` if actor is not running in the system
    ///
    /// `Some(ActorWrapper<A>)` if the actor is running on the system AND actor was created by the same builder or a clone of it
    ///
    /// `None` if actor is running on the system AND actor was not created by the same builder or a clone of it
    ///
    pub fn spawn<P>(&self, name: impl Into<String>, props: P) -> Option<ActorWrapper<A>>
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
            return self.system_state.get_actor_ref(actor_address);
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

        self.system_state.add_mailbox(actor_address.clone(), mailbox);
        self.wakeup_manager.add_sleeping_actor(
            actor_handler.get_address(),
            Arc::new(RwLock::new(actor_handler)),
        );

        self.existing.insert(actor_address, actor_ref.clone());
        Some(actor_ref)
    }
}
