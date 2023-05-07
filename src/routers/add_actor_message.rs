use serde::Serialize;
use std::hash::{Hash, Hasher};

use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorWrapper};

/// Adds an Actor to the Router
#[derive(Serialize)]
pub struct AddActorMessage<A>
where
    A: Actor,
{
    #[serde(skip)]
    pub actor: ActorWrapper<A>,
}

impl<A> AddActorMessage<A>
where
    A: Actor,
{
    pub fn new(actor: ActorWrapper<A>) -> Self {
        return Self { actor };
    }
}

impl<A> BaseActorMessage for AddActorMessage<A> where A: Actor {}

impl<A> Hash for AddActorMessage<A>
where
    A: Actor,
{
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
