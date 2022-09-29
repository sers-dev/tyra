use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::BaseActorMessage;
use crate::prelude::Actor;

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
