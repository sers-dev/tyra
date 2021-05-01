use crate::system::actor_system::ActorSystem;
use crate::actor::config::{Config, RestartPolicy};
use crate::config::tyractorsaur_config::DEFAULT_POOL;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::actor::Actor;
use std::panic::UnwindSafe;
use crate::actor::actor_factory::ActorFactory;

#[derive(Clone)]
pub struct Builder {
    system: ActorSystem,
    actor_config: Config,
}

impl Builder {
    pub fn new(system: ActorSystem, actor_name: String) -> Builder {
        let config = system.get_config();

        let actor_config = Config {
            actor_name,
            pool_name: String::from(DEFAULT_POOL),
            mailbox_size: config.global.default_mailbox_size,
            message_throughput: config.global.default_message_throughput,
            restart_policy: config.global.default_restart_policy,
        };

        Builder {
            system,
            actor_config,
        }
    }

    pub fn set_restart_policy(mut self, restart_policy: RestartPolicy) -> Builder {
        self.actor_config.restart_policy = restart_policy;
        self
    }

    pub fn set_pool_name(mut self, pool_name: impl Into<String>) -> Builder {
        self.actor_config.pool_name = pool_name.into();
        self
    }

    pub fn set_message_throughput(mut self, message_throughput: usize) -> Builder {
        self.actor_config.message_throughput = message_throughput;
        self
    }

    pub fn set_mailbox_unbounded(self) -> Builder {
        self.set_mailbox_size(0)
    }

    pub fn set_mailbox_size(mut self, mailbox_size: usize) -> Builder {
        self.actor_config.mailbox_size = mailbox_size;
        self
    }

    pub fn build<A, P>(&self, props: P) -> ActorWrapper<A>
    where
        A: Actor + UnwindSafe + 'static,
        P: ActorFactory<A> + 'static,
    {
        self.system.spawn(props, self.actor_config.clone())
    }
}