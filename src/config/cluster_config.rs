use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub hosts: Vec<String>,
}
