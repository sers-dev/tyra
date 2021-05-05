use crate::actor::actor::Actor;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::message::envelope::MessageEnvelope;
use crossbeam_channel::Sender;
use std::panic::UnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Mailbox<A> {
    pub is_stopped: Arc<AtomicBool>,
    pub is_sleeping: Arc<AtomicBool>,
    pub msg_in: Sender<MessageEnvelope<A>>,
}

impl<A> Clone for Mailbox<A>
where
    A: Actor + UnwindSafe,
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
