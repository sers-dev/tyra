use crate::actor::actor::Actor;
use crate::actor::actor_config::{ActorConfig, RestartPolicy};
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::config::tyractorsaur_config::DEFAULT_POOL;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;
use crossbeam_channel::{unbounded, bounded};
use crate::actor::mailbox::Mailbox;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use crate::actor::actor_address::ActorAddress;
use crate::actor::context::ActorContext;
use crate::actor::executor::{Executor, ExecutorTrait};
use crate::system::wakeup_manager::WakeupManager;
use crate::system::system_state::SystemState;

/// Used to create [Actor]s in the [ActorSystem]
#[derive(Clone)]
pub struct ActorBuilder {
    system: ActorSystem,
    system_state: SystemState,
    wakeup_manager: WakeupManager,
    actor_config: ActorConfig,
}

impl ActorBuilder {
    /// This is called through [ActorSystem.builder](../prelude/struct.ActorSystem.html#method.builder)
    pub fn new(system: ActorSystem, system_state: SystemState, wakeup_manager: WakeupManager, actor_name: String) -> ActorBuilder {
        let config = system.get_config();

        let actor_config = ActorConfig {
            actor_name,
            pool_name: String::from(DEFAULT_POOL),
            mailbox_size: config.general.default_mailbox_size,
            message_throughput: config.general.default_message_throughput,
            restart_policy: config.general.default_restart_policy,
        };

        ActorBuilder {
            system,
            system_state,
            wakeup_manager,
            actor_config,
        }
    }

    pub fn set_restart_policy(mut self, restart_policy: RestartPolicy) -> ActorBuilder {
        self.actor_config.restart_policy = restart_policy;
        self
    }

    pub fn set_pool_name(mut self, pool_name: impl Into<String>) -> ActorBuilder {
        self.actor_config.pool_name = pool_name.into();
        self
    }

    pub fn set_message_throughput(mut self, message_throughput: usize) -> ActorBuilder {
        self.actor_config.message_throughput = message_throughput;
        self
    }

    pub fn set_mailbox_unbounded(self) -> ActorBuilder {
        self.set_mailbox_size(0)
    }

    pub fn set_mailbox_size(mut self, mailbox_size: usize) -> ActorBuilder {
        self.actor_config.mailbox_size = mailbox_size;
        self
    }

    /// Creates the defined [Actor] on the [ActorSystem]
    pub fn build<A, P>(&self, props: P) -> ActorWrapper<A>
    where
        A: Actor + UnwindSafe + 'static,
        P: ActorFactory<A> + 'static,
    {
        self.spawn(props, self.actor_config.clone())
    }


    fn spawn<A, P>(&self, actor_props: P, actor_config: ActorConfig) -> ActorWrapper<A>
        where
            A: Actor + UnwindSafe + 'static,
            P: ActorFactory<A> + 'static,
    {
        let (sender, receiver) = if actor_config.mailbox_size == 0 {
            unbounded()
        } else {
            bounded(actor_config.mailbox_size)
        };

        let mailbox = Mailbox {
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            msg_in: sender,
        };
        let actor_address = ActorAddress {
            actor: actor_config.actor_name.clone(),
            system: String::from(self.system.get_name()),
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };
        let actor_ref = ActorWrapper::new(
            mailbox.clone(),
            actor_address.clone(),
            self.wakeup_manager.clone(),
        );

        let context = ActorContext {
            system: self.system.clone(),
            actor_ref: actor_ref.clone(),
        };
        let actor = actor_props.new_actor(context);
        let actor_handler = Executor::new(
            actor_props,
            actor_config,
            mailbox.clone(),
            receiver,
            self.system.clone(),
            String::from(self.system.get_name()),
            actor_ref.clone(),
        );

        self.system_state.add_actor(actor_address.clone(), Arc::new(actor));
        self.wakeup_manager.add_sleeping_actor(
            actor_handler.get_address(),
            Arc::new(RwLock::new(actor_handler)),
        );
        actor_ref
    }
}
