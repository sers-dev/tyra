use crate::actor::base_actor::BaseActor;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::message_type::MessageType;
use crate::message::system_stop_message::SystemStopMessage;
use std::any::{Any, TypeId};

pub trait MessageEnvelopeTrait<A>: Send + Sync
where
    A: BaseActor,
{
    fn handle(&mut self, actor: &mut A, context: &ActorContext<A>) -> MessageType;
}

pub struct MessageEnvelope<A>(Box<dyn MessageEnvelopeTrait<A> + Send + Sync>);

impl<A> MessageEnvelope<A> {
    pub fn new<M>(msg: M) -> Self
    where
        A: Handler<M> + BaseActor,
        M: ActorMessage + Send + Sync + 'static,
    {
        MessageEnvelope(Box::new(SyncMessageEnvelope { msg: Some(msg) }))
    }
}

impl<A> MessageEnvelopeTrait<A> for MessageEnvelope<A>
where
    A: BaseActor,
{
    fn handle(&mut self, act: &mut A, context: &ActorContext<A>) -> MessageType {
        self.0.handle(act, context)
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
    A: Handler<M> + BaseActor,
{
    fn handle(&mut self, act: &mut A, context: &ActorContext<A>) -> MessageType {
        if let Some(msg) = self.msg.take() {
            let msg_type_id = msg.type_id();
            act.handle(msg, context);
            if msg_type_id == TypeId::of::<ActorStopMessage>() {
                return MessageType::ActorStopMessage;
            } else if msg_type_id == TypeId::of::<SystemStopMessage>() {
                return MessageType::SystemStopMessage;
            }
        }
        MessageType::Other
    }
}
