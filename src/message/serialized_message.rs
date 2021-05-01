use crate::message::message::MessageTrait;

pub struct SerializedMessage {
    pub content: Vec<u8>,
}

impl MessageTrait for SerializedMessage {}