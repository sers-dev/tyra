pub mod actor;
pub mod actor_address;
pub mod actor_builder;
pub mod actor_config;
pub mod actor_factory;
pub mod actor_panic_source;
pub mod actor_result;
pub mod actor_send_error;
pub mod actor_state;
pub mod context;
pub mod executor;
pub mod handler;
pub mod mailbox;

pub mod prelude {
    pub use crate::actor::actor::Actor;
    pub use crate::actor::actor_address::ActorAddress;
    pub use crate::actor::actor_builder::ActorBuilder;
    pub use crate::actor::actor_factory::ActorFactory;
    pub use crate::actor::actor_panic_source::ActorPanicSource;
    pub use crate::actor::actor_result::ActorResult;
    pub use crate::actor::actor_send_error::ActorSendError;
    pub use crate::actor::context::ActorContext;
    pub use crate::actor::handler::Handler;
}
