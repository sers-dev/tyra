use std::hash::{Hash, Hasher};
use serde::Serialize;
use crate::message::actor_message::BaseActorMessage;

/// Wraps multiple [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
#[derive(Serialize)]
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

impl<M> Hash for BulkRouterMessage<M>
where
    M: BaseActorMessage + 'static
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}