use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorMessage, ActorWrapper};
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

/// Wraps an [ActorMessage](../prelude/trait.ActorMessage.html) to be sent at a later time
#[derive(Serialize)]
#[serde(bound(
serialize = "A: Actor",
deserialize = "A: Actor",
))]
pub struct DelayedMessage<A, M>
where
    M: BaseActorMessage + 'static,
    A: Actor,
{
    pub msg: M,
    pub destination: ActorWrapper<A>,
    pub delay: Duration,
    #[serde(skip)]
    pub started: Instant,
}

/// intentionally implements `ActorMessage`, because it does NOT provide a generic `Handler<ActorInitMessage>` implementation
impl<A, M> ActorMessage for DelayedMessage<A, M>
where
    M: BaseActorMessage + 'static,
    A: Actor,
{
}

impl<A, M> DelayedMessage<A, M>
where
    M: BaseActorMessage + 'static,
    A: Actor,
{
    pub fn new(msg: M, destination: ActorWrapper<A>, delay: Duration) -> Self {
        Self {
            msg,
            destination,
            delay,
            started: Instant::now(),
        }
    }
}

impl<A, M> Hash for DelayedMessage<A, M>
where
    M: BaseActorMessage + 'static,
    A: Actor,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.msg.hash(state);
    }
}
