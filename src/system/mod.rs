pub mod actor_system;
pub mod system_state;
mod thread_pool_manager;
pub mod wakeup_manager;
pub mod delay_actor;
pub mod internal_actor_manager;
pub mod actor_error;

pub mod prelude {
    pub use crate::system::actor_system::ActorSystem;
    pub use crate::system::actor_error::ActorError;

}
