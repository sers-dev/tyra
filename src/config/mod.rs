pub mod global_config;
pub mod pool_config;
pub mod tyra_config;

pub mod prelude {
    pub use crate::config::pool_config::ThreadPoolConfig;
    pub use crate::config::tyra_config::TyraConfig;
}
