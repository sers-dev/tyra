mod pool_config;
mod global_config;
mod tyractorsaur_config;

pub mod prelude {
    pub use crate::config::tyractorsaur_config::*;
    pub use crate::config::global_config::*;
    pub use crate::config::pool_config::*;
}