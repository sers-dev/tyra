use crate::message::actor_message::DefaultActorMessage;

pub struct ActorStopMessage {}

impl ActorStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for ActorStopMessage {}
