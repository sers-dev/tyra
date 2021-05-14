use crate::config::global_config::GeneralConfig;
use crate::config::pool_config::PoolConfig;

use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

pub const DEFAULT_POOL: &str = "default";

/// See [default.toml](https://github.com/sers-dev/tyractorsaur/blob/master/src/config/default.toml) for documentation of all configurations & their defaults
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TyractorsaurConfig {
    pub general: GeneralConfig,
    pub thread_pool: PoolConfig,
}

impl TyractorsaurConfig {
    /// Required for [ActorSystem.new](../prelude/struct.ActorSystem.html#method.new)
    ///
    /// Loads default config from [default.toml](https://github.com/sers-dev/tyractorsaur/blob/master/src/config/default.toml)
    /// Overwrites defaults through environment variables. Replace toml `.` with `_CONFIG_`, i.e. `TYRACTORSAUR_GENERAL_CONFIG_DEFAULT_MAILBOX_SIZE=1`
    ///
    /// Replaces `$HOSTNAME` with the actual hostname of the system for the `TYRACTORSAUR_GENERAL_CONFIG_NAME`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    ///
    /// use tyractorsaur::prelude::TyractorsaurConfig;
    ///
    /// let mut config = TyractorsaurConfig::new().unwrap();
    /// config.general.name = String::from("HelloWorld");
    /// ```
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
        if parsed.general.name == "$HOSTNAME" {
            parsed.general.name = String::from(hostname::get().unwrap().to_str().unwrap());
        }

        Ok(parsed)
    }
}
