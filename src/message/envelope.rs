use std::error::Error;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::prelude::{Actor, ActorResult};

pub trait MessageEnvelopeTrait<A>: Send + Sync
where
    A: Actor,
{
    fn handle(&mut self, actor: &mut A, context: &ActorContext<A>) -> Result<ActorResult, Box<dyn Error>>;
}

pub struct MessageEnvelope<A>(Box<dyn MessageEnvelopeTrait<A> + Send + Sync>);

impl<A> MessageEnvelope<A> {
    pub fn new<M>(msg: M) -> Self
    where
        A: Handler<M> + Actor,
        M: ActorMessage + Send + Sync + 'static,
    {
        MessageEnvelope(Box::new(SyncMessageEnvelope { msg: Some(msg) }))
    }
}

impl<A> MessageEnvelopeTrait<A> for MessageEnvelope<A>
where
    A: Actor,
{
    fn handle(&mut self, act: &mut A, context: &ActorContext<A>) -> Result<ActorResult, Box<dyn Error>> {
        return self.0.handle(act, context);
    }
}

pub struct SyncMessageEnvelope<M>
where
    M: ActorMessage + Send + Sync,
{
    msg: Option<M>,
}

impl<A, M> MessageEnvelopeTrait<A> for SyncMessageEnvelope<M>
where
    M: ActorMessage + Send + 'static,
    A: Handler<M> + Actor,
{
    fn handle(&mut self, act: &mut A, context: &ActorContext<A>) -> Result<ActorResult, Box<dyn Error>> {
        if let Some(msg) = self.msg.take() {
            return act.handle(msg, context);
        }
        return Ok(ActorResult::Ok);
    }
}
