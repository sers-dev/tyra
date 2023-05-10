use crate::config::global_config::GeneralConfig;
use crate::config::pool_config::PoolConfig;
use std::path::Path;

use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};

pub const DEFAULT_POOL: &str = "default";

/// See [default.toml](https://github.com/sers-dev/tyra/blob/master/src/config/default.toml) for documentation of all configurations & their defaults
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TyraConfig {
    pub general: GeneralConfig,
    pub thread_pool: PoolConfig,
}

impl TyraConfig {
    /// Required for [ActorSystem.new](../prelude/struct.ActorSystem.html#method.new)
    ///
    /// Loads default config from [default.toml](https://github.com/sers-dev/tyra/blob/master/src/config/default.toml)
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
    /// use tyra::prelude::TyraConfig;
    ///
    /// let mut config = TyraConfig::new().unwrap();
    /// config.general.name = String::from("HelloWorld");
    /// ```
    pub fn new() -> Result<Self, ConfigError> {
        let default: &str = std::include_str!("default.toml");

        let mut config = Config::builder();

        config = config.add_source(File::from_str(default, FileFormat::Toml));
        let path = Path::new("config/tyra.toml");
        if path.exists() {
            config = config.add_source(File::from(Path::new("config/tyra.toml")));
        }

        let conf = config.build().expect("Could not fetch Config");
        let mut parsed: TyraConfig = conf.try_deserialize().expect("Could not parse Config");
        if parsed.general.hostname == "$HOSTNAME" {
            parsed.general.hostname = String::from(hostname::get().unwrap().to_str().unwrap());
        }
        if parsed.general.name == "$CARGO_PKG_NAME" {
            parsed.general.name = option_env!("CARGO_PKG_NAME").unwrap_or("tyra").into();
        }

        Ok(parsed)
    }
}
