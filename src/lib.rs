#![allow(unused)]

mod actor;
mod actor_config;
mod actor_ref;
mod builder;
mod config;
mod context;
mod message;
mod system;

pub mod prelude {
    pub use crate::actor::*;
    pub use crate::actor_ref::*;
    pub use crate::config::prelude::*;
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
