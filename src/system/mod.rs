pub mod actor_system;
pub mod system_state;
mod thread_pool_manager;
pub mod wakeup_manager;

pub mod prelude {
    pub use crate::system::actor_system::ActorSystem;
}
