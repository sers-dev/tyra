use crate::message::actor_message::ActorMessage;

pub struct SystemStopMessage {}

impl SystemStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl ActorMessage for SystemStopMessage {}
