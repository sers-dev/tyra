use crate::message::actor_message::BaseActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
pub struct BulkRouterMessage<M>
where
    M: BaseActorMessage + 'static,
{
    pub data: Vec<M>,
}

impl<M> BaseActorMessage for BulkRouterMessage<M> where M: BaseActorMessage + 'static {}

impl<M> BulkRouterMessage<M>
where
    M: BaseActorMessage + 'static,
{
    pub fn new(data: Vec<M>) -> Self {
        Self { data }
    }
}
