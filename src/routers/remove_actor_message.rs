use std::hash::{Hash, Hasher};

use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorWrapper};

/// Removes an Actor from the Router
pub struct RemoveActorMessage<A>
where
    A: Actor,
{
    pub actor: ActorWrapper<A>,
}

impl<A> RemoveActorMessage<A>
where
    A: Actor,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        Self { actor }
    }
}

impl<A> BaseActorMessage for RemoveActorMessage<A> where A: Actor {}

impl<A> Hash for RemoveActorMessage<A>
    where
        A: Actor {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}