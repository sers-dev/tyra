use crate::actor::ActorTrait;
use crate::prelude::ActorRef;
use crate::system::ActorSystem;

#[derive(Clone)]
pub struct Context<A>
where
    A: ActorTrait + 'static,
{
    pub actor_ref: ActorRef<A>,
    pub system: ActorSystem,
}
