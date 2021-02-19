#![allow(unused)]

mod actor;
mod actor_ref;
mod builder;
mod config;
mod context;
mod handler;
mod message;
mod system;

pub mod prelude {
    pub use crate::actor::*;
    pub use crate::actor_ref::*;
    pub use crate::config::*;
    pub use crate::context::*;
    pub use crate::message::*;
    pub use crate::system::ActorSystem;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
