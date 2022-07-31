use std::time::{Duration, Instant};
use crate::message::actor_message::ActorMessage;
use crate::prelude::{Actor, ActorWrapper};

/// Wraps an [ActorMessage](../prelude/trait.ActorMessage.html) to be sent at a later time
pub struct DelayedMessage<A, M>
    where
        M: ActorMessage + 'static,
        A: Actor,
{
    pub msg: M,
    pub destination: ActorWrapper<A>,
    pub delay: Duration,
    pub started: Instant,
}

impl<A, M> ActorMessage for DelayedMessage<A, M>
where
    M: ActorMessage + 'static,
    A: Actor,
{}

impl<A, M> DelayedMessage<A, M>
    where
        M: ActorMessage + 'static,
        A: Actor,
{
    pub fn new(msg: M, destination: ActorWrapper<A>, delay: Duration) -> Self {
        Self { msg, destination, delay, started: Instant::now() }
    }
}
