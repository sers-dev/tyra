use crate::message::actor_message::DefaultActorMessage;

/// For Remote message handling
///
/// WARNING: This is a working POC implementation and you should definitely expect changes before a cluster module is released
///
/// Namely this will include switching to a versioned serialization format (i.e. Protobuf/Flatbuffers)
/// and it may also include some additional fields to make deserialization easier for end users
///
/// [ActorSystem.send_to_address](../prelude/struct.ActorSystem.html#method.send_to_address) uses this object to send serialized messages to Actors
#[derive(Hash)]
pub struct SerializedMessage {
    pub content: Vec<u8>,
}

impl SerializedMessage {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
}

impl DefaultActorMessage for SerializedMessage {}
