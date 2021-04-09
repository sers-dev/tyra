use crate::context::Context;
use crate::message::{MessageTrait, ActorStopMessage, SystemStopMessage};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::panic::UnwindSafe;

pub trait ActorTrait: Send + Sync + Sized + Clone + UnwindSafe {
    fn pre_start(&mut self, context: &Context<Self>) {}
    fn post_stop(&mut self, context: &Context<Self>) {}
    fn on_actor_stop(&mut self, context: &Context<Self>) {}
    fn on_system_stop(&mut self, context: &Context<Self>) {
        context.actor_ref.stop();
    }
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
        self.on_actor_stop(context);
    }
}

impl<A> Handler<SystemStopMessage> for A
    where
        A: ActorTrait + Sized,
{
    fn handle(&mut self, msg: SystemStopMessage, context: &Context<A>) {
        self.on_system_stop(context);
    }
}
