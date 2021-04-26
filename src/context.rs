use crate::actor::ActorTrait;
use crate::prelude::ActorRef;
use crate::system::ActorSystem;
use std::panic::UnwindSafe;


pub struct Context<A>
where
    Self: Send + Sync,
    A: ActorTrait + 'static,
{
    pub actor_ref: ActorRef<A>,
    pub system: ActorSystem,
}

impl<A> UnwindSafe for Context<A>
    where
        A: ActorTrait + 'static,
{}

impl<A> Clone for Context<A>
    where
        A: ActorTrait + 'static,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            actor_ref: self.actor_ref.clone(),
        }
    }
}