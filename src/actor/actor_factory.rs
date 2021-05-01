use crate::actor::actor::Actor;
use std::panic::UnwindSafe;
use crate::actor::context::Context;

pub trait ActorFactory<A>
    where
        A: Actor + UnwindSafe + 'static,
{
    fn new_actor(&self, context: Context<A>) -> A;
}
