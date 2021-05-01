use crate::system::ActorSystem;
use crate::actor::actor_config::{ActorConfig, RestartPolicy};
use crate::config::tyractorsaur_config::DEFAULT_POOL;
use crate::actor::actor_ref::ActorRef;
use crate::actor::actor::ActorTrait;
use std::panic::UnwindSafe;
use crate::actor::context::Context;

#[derive(Clone)]
pub struct ActorBuilder {
    system: ActorSystem,
    actor_config: ActorConfig,
}

impl ActorBuilder {
    pub fn new(system: ActorSystem, actor_name: String) -> ActorBuilder {
        let config = system.get_config();

        let actor_config = ActorConfig {
            actor_name,
            pool_name: String::from(DEFAULT_POOL),
            mailbox_size: config.global.default_mailbox_size,
            message_throughput: config.global.default_message_throughput,
            restart_policy: config.global.default_restart_policy,
        };

        ActorBuilder {
            system,
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

    pub fn set_mailbox_unbounded(mut self) -> ActorBuilder {
        self.set_mailbox_size(0)
    }

    pub fn set_mailbox_size(mut self, mailbox_size: usize) -> ActorBuilder {
        self.actor_config.mailbox_size = mailbox_size;
        self
    }

    pub fn build<A, P>(&self, props: P) -> ActorRef<A>
    where
        A: ActorTrait + UnwindSafe + 'static,
        P: ActorProps<A> + 'static,
    {
        self.system.spawn(props, self.actor_config.clone())
    }
}

pub trait ActorProps<A>
where
    A: ActorTrait + UnwindSafe + 'static,
{
    fn new_actor(&self, context: Context<A>) -> A;
}
