use crate::actor::actor_wrapper::ActorWrapper;
use crate::prelude::Actor;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;

/// Enables access to [ActorSystem] and [Actor] within  [Handler](./trait.Handler.html) implementations
pub struct ActorContext<A>
where
    Self: Send + Sync,
    A: Actor + 'static,
{
    pub actor_ref: ActorWrapper<A>,
    pub system: ActorSystem,
}

impl<A> UnwindSafe for ActorContext<A> where A: Actor + 'static {}

impl<A> Clone for ActorContext<A>
where
    A: Actor + 'static,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            actor_ref: self.actor_ref.clone(),
        }
    }
}
