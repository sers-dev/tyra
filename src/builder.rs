use crate::actor::{ActorTrait, Handler};
use crate::actor_ref::ActorRef;
use crate::message::MessageTrait;
use crate::system::{ActorSystem};
use std::sync::{Arc, RwLock};
use crate::config::prelude::DEFAULT_POOL;

#[derive(Clone)]
pub struct ActorBuilder {
    system: ActorSystem,
    pub name: String,
    pool: String,
    mailbox_size: usize,
    //pub actor: Arc<dyn ActorTrait>,
}

impl ActorBuilder {
    pub fn new(system: ActorSystem, name: String) -> ActorBuilder {
        ActorBuilder {
            system,
            name,
            pool: String::from(DEFAULT_POOL),
            mailbox_size: 0,
        }
    }

    pub fn set_pool(mut self, pool: impl Into<String>) -> ActorBuilder {
        self.pool = pool.into();
        self
    }

    pub fn set_mailbox_size(mut self, mailbox_size: usize) -> ActorBuilder {
        self.mailbox_size = mailbox_size;
        self
    }

    pub fn build<A>(&self, actor: A) -> ActorRef<A>
    where
        A: ActorTrait + Clone + 'static,
    {
        self.system
            .spawn(self.name.clone(), actor, self.mailbox_size, &self.pool)
    }
}
