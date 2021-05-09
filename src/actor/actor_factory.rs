use crate::actor::actor::Actor;
use crate::actor::context::ActorContext;
use std::panic::UnwindSafe;

/// [Actor]s can only be created from a Factory
pub trait ActorFactory<A>
where
    A: Actor + UnwindSafe + 'static,
{
    fn new_actor(&self, context: ActorContext<A>) -> A;
}
