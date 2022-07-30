use crate::actor::actor_wrapper::ActorWrapper;
use crate::prelude::BaseActor;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;

/// Enables access to [ActorSystem] and [Actor] within [Handler](./trait.Handler.html) implementations
///
/// Also injected into [ActorFactory.new_actor](../prelude/trait.ActorFactory.html#tymethod.new_actor), so that it can be stored within the Actor
pub struct ActorContext<A>
where
    Self: Send + Sync,
    A: BaseActor + 'static,
{
    pub actor_ref: ActorWrapper<A>,
    pub system: ActorSystem,
}

impl<A> UnwindSafe for ActorContext<A> where A: BaseActor + 'static {}

impl<A> Clone for ActorContext<A>
where
    A: BaseActor + 'static,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            actor_ref: self.actor_ref.clone(),
        }
    }
}
