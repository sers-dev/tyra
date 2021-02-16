use serde::{Serialize};
use crate::actor_ref::ActorRef;
use crate::actor::{ActorTrait, Handler};

pub trait MessageTrait: Send + Sync {}

pub trait MessageEnvelopeTrait<A>: Send + Sync
{
    fn handle(&mut self, actor: &mut A) {}
}

pub struct MessageEnvelope<A>(Box<dyn MessageEnvelopeTrait<A> + Send + Sync>);

impl<A> MessageEnvelope<A> {
    pub fn new<M>(msg: M) -> Self
        where
            A: Handler<M>,
            M: MessageTrait + Send + Sync + 'static,
    {
        MessageEnvelope(Box::new(SyncMessageEnvelope { msg: Some(msg) }))
    }
}


impl<A> MessageEnvelopeTrait<A> for MessageEnvelope<A> {
    fn handle(&mut self, act: &mut A) {
        self.0.handle(act)
    }
}

pub struct SyncMessageEnvelope<M>
    where
        M: MessageTrait + Send + Sync,
{
    msg: Option<M>,
}

impl<A, M> MessageEnvelopeTrait<A> for SyncMessageEnvelope<M>
    where
        M: MessageTrait + Send + 'static,
        A: Handler<M>,
{
    fn handle(&mut self, act: &mut A) {
        if let Some(msg) = self.msg.take() {
            act.handle( msg);
        }
    }
}
