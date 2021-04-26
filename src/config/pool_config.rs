use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadPoolConfig {
    pub actor_limit: usize,
    pub threads_min: usize,
    pub threads_max: usize,
    pub threads_factor: f32,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolConfig {
    pub config: HashMap<String, ThreadPoolConfig>,
}
