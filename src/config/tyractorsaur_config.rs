use crate::config::actor_config::ActorConfig;
use crate::config::pool_config::PoolConfig;

use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

pub const DEFAULT_POOL: &str = "default";
pub const SYSTEM_POOL: &str = "system";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TyractorsaurConfig {
    pub actor: ActorConfig,
    pub thread_pool: PoolConfig,
}

impl TyractorsaurConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        let default: &str = std::include_str!("default.toml");

        config
            .merge(File::from_str(default, FileFormat::Toml))
            .expect("Could not load default Config");

        config
            .merge(Environment::with_prefix("TYRACTORSAUR").separator("_CONFIG_"))
            .expect("Could not parse ENV variables");

        let mut parsed: TyractorsaurConfig = config.try_into().expect("Could not parse Config");
        if parsed.actor.name == "$HOSTNAME" {
            parsed.actor.name = String::from(hostname::get().unwrap().to_str().unwrap());
        }


        for (key, value) in parsed.thread_pool.config.iter_mut() {
            if key == DEFAULT_POOL {
                if value.size == 0 {
                    value.size = num_cpus::get() + (num_cpus::get() / 2);
                }
            } else if key == SYSTEM_POOL {
                if value.size == 0 {
                    value.size = num_cpus::get() / 2;
                }
            }
        }

        Ok(parsed)
    }
}