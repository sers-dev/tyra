use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemotingConfig {
    pub enabled: bool,
    pub bind: String,
}