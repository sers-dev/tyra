use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ActorAddress {
    pub remote: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}

impl ActorAddress {
    pub fn new(remote: impl Into<String>, system: impl Into<String>, pool: impl Into<String>, actor: impl Into<String>) -> Self {
        return Self {
            remote: remote.into(),
            system: system.into(),
            pool: pool.into(),
            actor: actor.into(),
        }
    }
}