use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::Sender;
use crate::message::envelope::MessageEnvelope;
use crate::actor::{ActorTrait, Handler};
use std::panic::UnwindSafe;
use crate::message::message::MessageTrait;

pub struct Mailbox<A> {
    pub is_stopped: Arc<AtomicBool>,
    pub is_sleeping: Arc<AtomicBool>,
    pub msg_in: Sender<MessageEnvelope<A>>,
}

impl<A> Clone for Mailbox<A>
    where
        A: ActorTrait + UnwindSafe,
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
    A: ActorTrait,
{
    pub fn send<M>(&self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait + 'static,
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