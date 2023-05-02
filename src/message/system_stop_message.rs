use serde::{Deserialize, Serialize};
use crate::message::actor_message::DefaultActorMessage;

#[derive(Hash, Serialize, Deserialize)]
pub struct SystemStopMessage {}

impl SystemStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for SystemStopMessage {}
