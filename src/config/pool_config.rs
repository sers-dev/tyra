use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadPoolConfig {
    pub actor_limit: usize,
    pub thread_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolConfig {
    pub config: HashMap<String, ThreadPoolConfig>
}

