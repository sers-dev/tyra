use crate::message::actor_message::ActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
pub struct BulkRouterMessage<M>
where
    M: ActorMessage + 'static,
{
    pub data: Vec<M>,
}

impl<M> ActorMessage for BulkRouterMessage<M> where M: ActorMessage + 'static {}

impl<M> BulkRouterMessage<M>
where
    M: ActorMessage + 'static,
{
    pub fn new(data: Vec<M>) -> Self {
        Self { data }
    }
}
