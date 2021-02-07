use crate::actor::{ActorTrait, Handler};
use std::sync::{Arc, RwLock};
use crate::message::MessageTrait;

pub struct ActorRef<A>
    where
        A: ActorTrait {
    actor: Arc<RwLock<A>>,
}

impl<A> ActorRef<A>
    where
        A: ActorTrait,
{
    pub fn new(actor: Arc<RwLock<A>>) -> Self {
        Self {
            actor
        }
    }
    pub fn send<M>(&mut self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait
    {
        let mut actor = self.actor.write().unwrap();
        actor.handle(msg);
        println!("AAAAAAAAAAAAAAA")
    }
}