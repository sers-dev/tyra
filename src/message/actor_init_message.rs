use serde::{Deserialize, Serialize};
use crate::prelude::ActorMessage;

/// Can be implemented by an Actor through Handler<ActorInitMessage> to be used to init an Actor
#[derive(Hash, Serialize, Deserialize)]
pub struct ActorInitMessage {}

impl ActorInitMessage {
    pub fn new() -> Self {
        Self {}
    }
}

/// intentionally implements `ActorMessage`, because it does NOT provide a generic `Handler<ActorInitMessage>` implementation
impl ActorMessage for ActorInitMessage {}
