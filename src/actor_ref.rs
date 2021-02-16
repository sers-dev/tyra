use crate::actor::{ActorTrait, Handler};
use std::sync::{Arc, RwLock};
use crate::message::{MessageTrait, MessageEnvelopeTrait, MessageEnvelope};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::borrow::BorrowMut;
use std::any::Any;
use std::ops::DerefMut;

pub trait ActorRefTrait: Send + Sync {
    fn handle(&self);

}

#[derive(Clone)]
pub struct ActorRef<A>
    where
        A: ActorTrait,
{
    actor: Arc<RwLock<A>>,
    mailbox_in: Sender<MessageEnvelope<A>>,
    mailbox_out: Receiver<MessageEnvelope<A>>,
}

impl<A> ActorRefTrait for ActorRef<A>
where
    A: ActorTrait + Clone + 'static
{
    fn handle(&self) {
        let mut msg = self.mailbox_out.recv().unwrap();
        let mut a = self.actor.write().unwrap();
        let mut ac = a.deref_mut();
        msg.handle(ac);

    }
}

impl<A> ActorRef<A>
    where
        A: ActorTrait,
{
    pub fn new(actor: Arc<RwLock<A>>, sender: Sender<MessageEnvelope<A>>, receiver: Receiver<MessageEnvelope<A>>) -> Self {
        Self {
            actor,
            mailbox_in: sender,
            mailbox_out: receiver
        }
    }
    pub fn send<M>(&mut self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait + Clone + 'static
    {
        let abcd = msg.clone();
        let defg = msg.clone();

        self.mailbox_in.send(MessageEnvelope::new(abcd));
        let mut actor = self.actor.clone();
    }

    pub fn handle_generic<F>(actor: Arc<dyn ActorTrait>, msg: Arc<dyn MessageTrait>, func: F)
    where
        F: Fn(Arc<dyn ActorTrait>, Arc<dyn MessageTrait>),
    {
        func(actor, msg)
    }
}