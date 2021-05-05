use crate::actor::actor::Actor;
use crate::actor::context::Context;
use std::panic::UnwindSafe;

pub trait ActorFactory<A>
where
    A: Actor + UnwindSafe + 'static,
{
    fn new_actor(&self, context: Context<A>) -> A;
}
