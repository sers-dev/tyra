use crate::message::actor_message::DefaultActorMessage;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Puts an actor to sleep for a specified time
#[derive(Hash, Serialize, Deserialize)]
pub struct SleepMessage {
    pub duration: Duration,
}

impl DefaultActorMessage for SleepMessage {}

impl SleepMessage {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}
