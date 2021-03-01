use crate::actor::{ActorTrait, Handler};
use crate::actor_config::ActorConfig;
use crate::actor_ref::ActorRef;
use crate::config::prelude::DEFAULT_POOL;
use crate::message::MessageTrait;
use crate::system::ActorSystem;
use std::sync::{Arc, RwLock};

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
        };

        ActorBuilder {
            system,
            actor_config,
        }
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

    pub fn build<A>(&self, actor: A) -> ActorRef<A>
    where
        A: ActorTrait + Clone + 'static,
    {
        self.system.spawn(actor, self.actor_config.clone())
    }
}
