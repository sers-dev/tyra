use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub name: String,
    pub hostname: String,
    pub default_mailbox_size: usize,
    pub default_message_throughput: usize,
    pub override_panic_hook: bool,
    pub enable_signal_handling: bool,
    pub graceful_timeout_in_seconds: u64,
}
