use crate::message::actor_message::{DefaultActorMessage};

#[derive(Hash)]
pub struct ActorStopMessage {}

impl ActorStopMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultActorMessage for ActorStopMessage {}
