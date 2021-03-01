mod pool_config;
mod actor_config;
mod tyractorsaur_config;

pub mod prelude {
    pub use crate::config::tyractorsaur_config::*;
    pub use crate::config::actor_config::*;
    pub use crate::config::pool_config::*;
}