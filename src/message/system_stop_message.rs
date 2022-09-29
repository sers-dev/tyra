use crate::message::actor_message::DefaultActorMessage;

pub struct SystemStopMessage {}

impl SystemStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for SystemStopMessage {}
