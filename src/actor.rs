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

pub struct ActorRef<A>
where
    A: ActorTrait {
    actor: Arc<A>,
}

impl<A> ActorRef<A>
where
    A: ActorTrait,
{
    pub fn new(actor: Arc<A>) -> Self {
        Self {
            actor
        }
    }
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait
    {
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