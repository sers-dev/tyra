use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// See [default.toml](https://github.com/sers-dev/tyra/blob/master/src/config/default.toml) for documentation of all configurations & their defaults
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadPoolConfig {
    pub actor_limit: usize,
    pub threads_min: usize,
    pub threads_max: usize,
    pub threads_factor: f32,
}

impl ThreadPoolConfig {
    /// Required for [ActorSystem.add_pool_with_config](../prelude/struct.ActorSystem.html#method.add_pool_with_config)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use tyra::prelude::ThreadPoolConfig;
    ///
    /// let config = ThreadPoolConfig::new(0, 1, 2, 1.0);
    /// ```
    pub fn new(
        actor_limit: usize,
        threads_min: usize,
        threads_max: usize,
        threads_factor: f32,
    ) -> Self {
        Self {
            actor_limit,
            threads_min,
            threads_max,
            threads_factor,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolConfig {
    pub config: HashMap<String, ThreadPoolConfig>,
}
