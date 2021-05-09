use crate::message::actor_message::ActorMessage;

/// For Remote message handling
pub struct SerializedMessage {
    pub content: Vec<u8>,
}

impl ActorMessage for SerializedMessage {}
