use crate::actor::{ActorTrait, Handler};
use crate::actor_ref::ActorHandler;
use serde::Serialize;
use std::any::{TypeId, Any};
use crate::context::Context;

pub trait MessageTrait: Send + Sync {}

pub trait MessageEnvelopeTrait<A>: Send + Sync
where
    A: ActorTrait
{
    fn handle(&mut self, actor: &mut A, context: &Context<A>) -> MessageType;
}

pub struct MessageEnvelope<A>(Box<dyn MessageEnvelopeTrait<A> + Send + Sync>);

impl<A> MessageEnvelope<A> {
    pub fn new<M>(msg: M) -> Self
    where
        A: Handler<M> + ActorTrait,
        M: MessageTrait + Send + Sync + 'static,
    {
        MessageEnvelope(Box::new(SyncMessageEnvelope { msg: Some(msg) }))
    }
}

impl<A> MessageEnvelopeTrait<A> for MessageEnvelope<A>
where
    A: ActorTrait
{
    fn handle(&mut self, act: &mut A, context: &Context<A>) -> MessageType {
        self.0.handle(act, context)
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
    A: Handler<M> + ActorTrait,
{
    fn handle(&mut self, act: &mut A, context: &Context<A>) -> MessageType {
        if let Some(msg) = self.msg.take() {
            let msg_type_id = msg.type_id();
            act.handle(msg, context);
            if msg_type_id == TypeId::of::<StopMessage>() {
                return MessageType::StopMessage
            }
        }
        MessageType::Unknown
    }
}

#[derive(Clone)]
pub struct StopMessage {}

impl MessageTrait for StopMessage {}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MessageType {
    Unknown,
    StopMessage
}