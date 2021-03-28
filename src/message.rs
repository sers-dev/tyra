use crate::actor::{ActorTrait, Handler};
use crate::actor_ref::ActorHandler;
use serde::Serialize;
use std::any::{TypeId, Any};

pub trait MessageTrait: Send + Sync {}

pub trait MessageEnvelopeTrait<A>: Send + Sync {
    fn handle(&mut self, actor: &mut A) -> MessageType;
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
    fn handle(&mut self, act: &mut A) -> MessageType {
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
    fn handle(&mut self, act: &mut A) -> MessageType {
        if let Some(msg) = self.msg.take() {
            let msg_type_id = msg.type_id();
            act.handle(msg);
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