use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadPoolConfig {
    pub message_throughput: usize,
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolConfig {
    pub config: HashMap<String, ThreadPoolConfig>
}

