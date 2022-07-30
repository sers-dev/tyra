use crate::actor::base_actor::BaseActor;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::ActorMessage;

/// Adds an Actor to the Router
pub struct AddActorMessage<A>
where
    A: BaseActor + 'static,
{
    pub actor: ActorWrapper<A>,
}

impl<A> AddActorMessage<A>
where
    A: BaseActor + 'static,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        Self { actor }
    }
}

impl<A> ActorMessage for AddActorMessage<A> where A: BaseActor + 'static {}
