use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorMessage, ActorWrapper};
use std::time::{Duration, Instant};

/// Wraps an [ActorMessage](../prelude/trait.ActorMessage.html) to be sent at a later time
pub struct DelayedMessage<A, M>
where
    M: BaseActorMessage + 'static,
    A: Actor,
{
    pub msg: M,
    pub destination: ActorWrapper<A>,
    pub delay: Duration,
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
