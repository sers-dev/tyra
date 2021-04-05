use crate::context::Context;
use crate::message::{MessageTrait, StopMessage};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::{Arc, RwLock};

pub trait ActorTrait: Send + Sync {
    fn pre_start(&mut self) {}
    fn post_stop(&mut self) {}
}

pub trait Handler<M: ?Sized>
where
    Self: ActorTrait + Sized,
    M: MessageTrait,
{
    fn handle(&mut self, msg: M, context: &Context<Self>);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ActorAddress {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}

impl<A> Handler<StopMessage> for A
where
    A: ActorTrait + Sized
{
    fn handle(&mut self, msg: StopMessage, context: &Context<A>) {}
}

