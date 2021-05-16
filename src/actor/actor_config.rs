use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActorConfig {
    //pub actor_name: String,
    pub pool_name: String,
    pub mailbox_size: usize,
    pub message_throughput: usize,
    pub restart_policy: RestartPolicy,
}

/// Defines behavior of [Actor](../prelude/trait.Actor.html) in case of a panic when handling a message
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum RestartPolicy {
    Never,
    Always,
}
