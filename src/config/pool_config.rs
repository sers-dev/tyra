use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configures thread pools
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadPoolConfig {
    pub actor_limit: usize,
    pub threads_min: usize,
    pub threads_max: usize,
    pub threads_factor: f32,
}

impl ThreadPoolConfig {
    pub fn new(actor_limit: usize, threads_min: usize, threads_max: usize, threads_factor: f32) -> Self {
        Self {
            actor_limit,
            threads_min,
            threads_max,
            threads_factor,
        }
    }
}

/// Map of [ThreadPoolConfig]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolConfig {
    pub config: HashMap<String, ThreadPoolConfig>,
}
