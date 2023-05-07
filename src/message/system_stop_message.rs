use crate::message::actor_message::DefaultActorMessage;
use serde::{Deserialize, Serialize};

#[derive(Hash, Serialize, Deserialize)]
pub struct SystemStopMessage {}

impl SystemStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for SystemStopMessage {}
