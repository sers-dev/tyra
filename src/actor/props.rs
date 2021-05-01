use crate::actor::actor::ActorTrait;
use std::panic::UnwindSafe;
use crate::actor::context::Context;

pub trait ActorProps<A>
    where
        A: ActorTrait + UnwindSafe + 'static,
{
    fn new_actor(&self, context: Context<A>) -> A;
}
