use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::ActorMessage;
use crate::prelude::Actor;

/// Removes an Actor from the Router
pub struct RemoveActorMessage<A>
where
    A: Actor + 'static,
{
    pub actor: ActorWrapper<A>,
}

impl<A> RemoveActorMessage<A>
where
    A: Actor + 'static,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        Self { actor }
    }
}

impl<A> ActorMessage for RemoveActorMessage<A> where A: Actor + 'static {}
