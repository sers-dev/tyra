use crate::message::actor_message::ActorMessage;

pub struct SerializedMessage {
    pub content: Vec<u8>,
}

impl ActorMessage for SerializedMessage {}