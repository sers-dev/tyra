use crate::message::MessageTrait;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use std::sync::{Arc, RwLock};

pub trait ActorTrait: Send + Sync {
}

pub trait Handler<M>
where
    Self: ActorTrait,
    M: MessageTrait
{
    fn handle(&mut self, msg: M);
}

struct Envelope {
    content: dyn MessageTrait
}

pub struct ActorRef<A>
where
    A: ActorTrait {
    actor: Arc<RwLock<A>>,
}

impl<A> ActorRef<A>
where
    A: ActorTrait,
{
    pub fn new(actor: Arc<RwLock<A>>) -> Self {
        Self {
            actor
        }
    }
    pub fn send<M>(&mut self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait
    {
        let mut actor = self.actor.write().unwrap();
        actor.handle(msg);
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