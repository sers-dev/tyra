use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorConfig {
    pub pool_name: String,
    pub mailbox_size: usize,
    pub message_throughput: usize,
}
