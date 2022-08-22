use std::time::Duration;
use crate::message::actor_message::ActorMessage;

/// Puts an actor to sleep for a specified time
pub struct SleepMessage {
    pub duration: Duration,
}

impl ActorMessage for SleepMessage {}

impl SleepMessage
{
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

