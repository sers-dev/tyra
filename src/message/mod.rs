pub mod actor_message;
pub mod actor_stop_message;
pub mod bulk_actor_message;
pub mod envelope;
pub mod serialized_message;
pub mod system_stop_message;
pub mod delayed_message;
pub mod sleep_message;
pub mod actor_init_message;
pub mod prelude {
    pub use crate::message::actor_message::ActorMessage;
    pub use crate::message::bulk_actor_message::BulkActorMessage;
    pub use crate::message::serialized_message::SerializedMessage;
    pub use crate::message::actor_init_message::ActorInitMessage;
}
