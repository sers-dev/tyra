use crate::actor::actor::Actor;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::message::actor_message::ActorMessage;

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
