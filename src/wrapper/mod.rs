mod local_actor_wrapper;
mod remote_actor_wrapper;
pub mod actor_wrapper;

pub mod prelude {
    pub use crate::wrapper::actor_wrapper::ActorWrapper;
}