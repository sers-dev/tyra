mod actor;
mod context;
mod system;
mod message;
mod config;
mod handler;


pub mod prelude {
    pub use crate::system::*;
    pub use crate::actor::*;
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