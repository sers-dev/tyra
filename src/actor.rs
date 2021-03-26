use crate::context::Context;
use crate::message::MessageTrait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::{Arc, RwLock};

pub trait ActorTrait: Send + Sync {
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized + 'static,
    {
        self
    }
}

pub trait Handler<M: ?Sized>
where
    Self: ActorTrait,
    M: MessageTrait,
{
    fn handle(&mut self, msg: M);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ActorAddress {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}
