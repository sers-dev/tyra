use serde::{Deserialize, Serialize};
use crate::message::actor_message::{DefaultActorMessage};

#[derive(Hash, Serialize, Deserialize)]
pub struct ActorStopMessage {}

impl ActorStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for ActorStopMessage {}
