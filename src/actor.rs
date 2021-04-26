use crate::context::Context;
use crate::message::{MessageTrait, ActorStopMessage, SystemStopMessage, SerializedMessage};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::panic::UnwindSafe;

pub trait ActorTrait: Send + Sync + UnwindSafe {
    fn pre_start(&mut self) {}
    fn post_stop(&mut self) {}
    fn on_actor_stop(&mut self) {}
    fn on_system_stop(&mut self) {}
    fn handle_serialized_message(&self, msg: SerializedMessage) {}
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

impl<A> Handler<ActorStopMessage> for A
where
    A: ActorTrait + Sized,
{
    fn handle(&mut self, msg: ActorStopMessage, context: &Context<A>) {
        self.on_actor_stop();
    }
}

impl<A> Handler<SystemStopMessage> for A
    where
        A: ActorTrait + Sized,
{
    fn handle(&mut self, msg: SystemStopMessage, context: &Context<A>) {
        self.on_system_stop();
    }
}
