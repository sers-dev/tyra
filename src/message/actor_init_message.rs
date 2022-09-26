use crate::message::actor_message::ActorMessage;

/// Can be implemented by an Actor through Handler<ActorInitMessage> to be used to init an Actor
pub struct ActorInitMessage {}

impl ActorInitMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl ActorMessage for ActorInitMessage {}
