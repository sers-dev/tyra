use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub name: String,
    pub hostname: String,
    pub default_mailbox_size: usize,
    pub default_message_throughput: usize,
    pub override_panic_hook: bool,
    pub signal_graceful_timeout_in_seconds: usize,
}
