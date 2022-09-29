/// Bulk Actor Message, that can wrap and send multiple [ActorMessage](../prelude/trait.ActorMessage.html) at once
///
use crate::message::actor_message::BaseActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to an Actor
pub struct BulkActorMessage<M>
where
    M: BaseActorMessage + 'static,
{
    pub data: Vec<M>,
}

impl<M> BaseActorMessage for BulkActorMessage<M> where M: BaseActorMessage + 'static {}

impl<M> BulkActorMessage<M>
where
    M: BaseActorMessage + 'static,
{
    pub fn new(data: Vec<M>) -> Self {
        Self { data }
    }
}
