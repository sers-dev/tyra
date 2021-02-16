#![allow(unused)]

mod actor;
mod context;
mod system;
mod message;
mod config;
mod handler;
mod builder;
mod actor_ref;


pub mod prelude {
    pub use crate::system::ActorSystem;
    pub use crate::actor::*;
    pub use crate::actor_ref::*;
    pub use crate::context::*;
    pub use crate::message::*;
    pub use crate::config::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}