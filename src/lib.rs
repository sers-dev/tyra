#![allow(unused)]

mod actor;
mod actor_config;
mod actor_ref;
mod builder;
mod config;
mod context;
mod system;
mod router;
mod message;
mod mailbox;
mod actor_handler;
mod actor_state;

pub mod prelude {
    pub use crate::actor::*;
    pub use crate::actor_ref::*;
    pub use crate::config::prelude::*;
    pub use crate::context::*;
    pub use crate::system::ActorSystem;
    pub use crate::builder::ActorProps;
    pub use crate::router::prelude::*;
    pub use crate::message::prelude::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
