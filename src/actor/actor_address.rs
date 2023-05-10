use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ActorAddress {
    pub hostname: String,
    pub system: String,
    pub pool: String,
    pub actor: String,
}

impl ActorAddress {
    pub fn new(
        hostname: impl Into<String>,
        system: impl Into<String>,
        pool: impl Into<String>,
        actor: impl Into<String>,
    ) -> Self {
        return Self {
            hostname: hostname.into(),
            system: system.into(),
            pool: pool.into(),
            actor: actor.into(),
        };
    }
}
