use crate::message::MessageTrait;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use std::sync::{Arc, RwLock};
use std::any::Any;

pub trait ActorTrait: Send + Sync {
    fn as_any(&self) -> &dyn Any
        where
            Self: Sized + 'static
    {
        self
    }
}

pub trait Handler<M: ?Sized>
where
    Self: ActorTrait,
    M: MessageTrait
{
    fn handle(&mut self, msg: M);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorAddress {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}