use crate::message::MessageTrait;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use std::sync::Arc;

pub trait ActorTrait: Send + Sync {
}

pub trait Handler<M>
where
    Self: ActorTrait,
    M: MessageTrait
{
    fn handle(&self, msg: M);
}

struct Envelope {
    content: dyn MessageTrait
}

pub struct ActorRef<M>
where
    M: MessageTrait {
    actor: Arc<dyn Handler<M>>,
}

impl<M> ActorRef<M>
where
    M: MessageTrait {
    pub fn new(actor: Arc<dyn Handler<M>>) -> Self {
        Self {
            actor
        }
    }
    pub fn send(&self, msg: M) {
        self.actor.handle(msg);
        println!("AAAAAAAAAAAAAAA")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorAddress {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}