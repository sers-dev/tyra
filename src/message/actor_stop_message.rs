use crate::message::actor_message::ActorMessage;

pub struct ActorStopMessage {}

impl ActorStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl ActorMessage for ActorStopMessage {}
