use config::{Config, File, Environment, FileFormat, ConfigError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorConfig {
    pub name: String,
    pub default_thread_pool_size: usize,
    pub system_thread_pool_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TractorConfig {
    pub actor: ActorConfig,
}


impl TractorConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        let default: &str = std::include_str!("default.toml");

        config.merge(File::from_str(default, FileFormat::Toml)).expect("Could not load default Config");

        config.merge(Environment::with_prefix("TRACTOR").separator("_CONFIG_")).expect("Could not parse ENV variables");

        let mut parsed :TractorConfig = config.try_into().expect("Could not parse Config");
        if parsed.actor.name == "$HOSTNAME" {
            parsed.actor.name = String::from(hostname::get().unwrap().to_str().unwrap());
        }
        if parsed.actor.default_thread_pool_size == 0 {
            parsed.actor.default_thread_pool_size = num_cpus::get() + (num_cpus::get() / 2);
        }
        if parsed.actor.system_thread_pool_size == 0 {
            parsed.actor.system_thread_pool_size = num_cpus::get() / 2;
        }

        Ok(parsed)
    }

}