use crate::message::actor_message::ActorMessage;

/// For Remote message handling
///
/// WARNING: This is a working POC implementation and you should definitely expect changes before the 1.0.0 Release.
///
/// Namely this will include switching to a versioned serialization format (i.e. Protobuf/Flatbuffers)
/// and it may also include some additional fields to make deserialization easier for Endusers
///
/// [ActorSystem.send_to_address](../prelude/struct.ActorSystem.html#method.send_to_address) uses this object to send serialized messages to Actors
pub struct SerializedMessage {
    pub content: Vec<u8>,
}

impl SerializedMessage {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content
        }
    }
}

impl ActorMessage for SerializedMessage {}
