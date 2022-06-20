/// Bulk Actor Message, that can wrap and send multiple [ActorMessage](../prelude/trait.ActorMessage.html) at once
///
use crate::message::actor_message::ActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to an Actor
pub struct BulkActorMessage<M>
    where
        M: ActorMessage + 'static,
{
    pub data: Vec<M>,
}

impl<M> ActorMessage for BulkActorMessage<M> where M: ActorMessage + 'static {}

impl<M> BulkActorMessage<M>
    where
        M: ActorMessage + 'static,
{
    pub fn new(data: Vec<M>) -> Self {
        Self { data }
    }
}
