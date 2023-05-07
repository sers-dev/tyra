use crate::message::actor_message::DefaultActorMessage;
use serde::{Deserialize, Serialize};

#[derive(Hash, Serialize, Deserialize)]
pub struct ActorStopMessage {}

impl ActorStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for ActorStopMessage {}
