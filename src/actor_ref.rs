use crate::actor::{ActorTrait, Handler};
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use crate::actor_config::ActorConfig;

pub trait ActorRefTrait: Send + Sync {
    fn handle(&self);
    fn get_config(&self) -> &ActorConfig;
}

#[derive(Clone)]
pub struct ActorRef<A>
where
    A: ActorTrait,
{
    actor: Arc<RwLock<A>>,
    actor_config: ActorConfig,
    mailbox_in: Sender<MessageEnvelope<A>>,
    mailbox_out: Receiver<MessageEnvelope<A>>,
}

impl<A> ActorRefTrait for ActorRef<A>
where
    A: ActorTrait + Clone + 'static,
{
    fn handle(&self) {
        let mut msg = self.mailbox_out.recv().unwrap();
        let mut a = self.actor.write().unwrap();
        let mut ac = a.deref_mut();
        msg.handle(ac);
    }

    fn get_config(&self) -> &ActorConfig {
        &self.actor_config
    }
}

impl<A> ActorRef<A>
where
    A: ActorTrait,
{
    pub fn new(
        actor: Arc<RwLock<A>>,
        actor_config: ActorConfig,
        sender: Sender<MessageEnvelope<A>>,
        receiver: Receiver<MessageEnvelope<A>>,
    ) -> Self {
        Self {
            actor,
            actor_config,
            mailbox_in: sender,
            mailbox_out: receiver,
        }
    }
    pub fn send<M>(&mut self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
    {
        self.mailbox_in.send(MessageEnvelope::new(msg));
    }

}
