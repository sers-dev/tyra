use crate::message::actor_message::BaseActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
pub struct SendToAllTargetsMessage<M>
    where
        M: BaseActorMessage + 'static,
{
    pub msg: M,
}

impl<M> BaseActorMessage for SendToAllTargetsMessage<M> where M: BaseActorMessage + 'static {}

impl<M> SendToAllTargetsMessage<M>
    where
        M: BaseActorMessage + 'static,
{
    pub fn new(msg: M) -> Self {
        Self { msg }
    }
}
