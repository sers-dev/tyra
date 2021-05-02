pub mod actor_stop_message;
pub mod actor_message;
pub mod message_type;
pub mod system_stop_message;
pub mod serialized_message;
pub mod envelope;

pub mod prelude {
    pub use crate::message::actor_message::ActorMessage;
    pub use crate::message::serialized_message::SerializedMessage;
}
