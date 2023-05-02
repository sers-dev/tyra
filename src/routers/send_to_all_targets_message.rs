use std::hash::{Hash, Hasher};
use serde::Serialize;
use crate::message::actor_message::BaseActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
#[derive(Serialize)]
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

impl<M> Hash for SendToAllTargetsMessage<M>
    where
        M: BaseActorMessage + 'static
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.msg.hash(state);
    }
}