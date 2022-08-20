use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::message::envelope::MessageEnvelope;
use crate::prelude::{Actor, SerializedMessage};
use crossbeam_channel::Sender;
use std::any::Any;
use std::panic::UnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub trait BaseMailbox: Send + Sync + UnwindSafe {
    fn send_serialized(&self, _msg: SerializedMessage);
    fn as_any(&self) -> &dyn Any;
    fn is_sleeping(&self) -> bool;
}

pub struct Mailbox<A> {
    pub is_stopped: Arc<AtomicBool>,
    pub is_sleeping: Arc<AtomicBool>,
    pub msg_in: Sender<MessageEnvelope<A>>,
}

impl<A> BaseMailbox for Mailbox<A>
where
    A: Handler<SerializedMessage> + 'static,
{
    fn send_serialized(&self, msg: SerializedMessage) {
        self.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Relaxed)
    }
}

impl<A> Clone for Mailbox<A>
where
    A: Actor,
{
    fn clone(&self) -> Self {
        Self {
            msg_in: self.msg_in.clone(),
            is_stopped: self.is_stopped.clone(),
            is_sleeping: self.is_sleeping.clone(),
        }
    }
}

impl<A> Mailbox<A>
where
    A: Actor,
{
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: ActorMessage + 'static,
    {
        self.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }

    pub fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Relaxed)
    }

    pub fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }
}
