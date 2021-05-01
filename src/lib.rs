mod config;
mod system;
mod router;
mod message;
mod actor;

pub mod prelude {
    pub use crate::actor::prelude::*;
    pub use crate::config::prelude::*;
    pub use crate::message::prelude::*;
    pub use crate::router::prelude::*;
    pub use crate::system::ActorSystem;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
