use crate::actor::actor::Actor;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::ActorMessage;

/// Adds an Actor to the Router
pub struct AddActorMessage<A>
where
    A: Actor + 'static,
{
    pub actor: ActorWrapper<A>,
}

impl<A> AddActorMessage<A>
where
    A: Actor + 'static,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        Self { actor }
    }
}

impl<A> ActorMessage for AddActorMessage<A> where A: Actor + 'static {}
