use crate::actor::base_actor::BaseActor;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::ActorMessage;

/// Removes an Actor from the Router
pub struct RemoveActorMessage<A>
where
    A: BaseActor + 'static,
{
    pub actor: ActorWrapper<A>,
}

impl<A> RemoveActorMessage<A>
where
    A: BaseActor + 'static,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        Self { actor }
    }
}

impl<A> ActorMessage for RemoveActorMessage<A> where A: BaseActor + 'static {}
