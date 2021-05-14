pub mod global_config;
pub mod pool_config;
pub mod tyractorsaur_config;

pub mod prelude {
    pub use crate::config::pool_config::ThreadPoolConfig;
    pub use crate::config::tyractorsaur_config::TyractorsaurConfig;
}
