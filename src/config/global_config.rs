use crate::actor::actor_config::RestartPolicy;
use serde::{Deserialize, Serialize};

/// Configures System name and defaults
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub name: String,
    pub default_mailbox_size: usize,
    pub default_message_throughput: usize,
    pub default_restart_policy: RestartPolicy,
}
