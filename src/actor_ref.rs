use crate::actor::{ActorTrait, Handler};
use crate::actor_config::ActorConfig;
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::panic::{UnwindSafe, AssertUnwindSafe, catch_unwind};
use crate::actor_ref::HandleResult::{MailboxEmpty, Success, ActorPanic};

pub trait ActorRefTrait: Send + Sync {
    fn handle(&self) -> HandleResult;
    fn get_config(&self) -> &ActorConfig;
    fn reset(&mut self);
}

#[derive(PartialEq)]
pub enum HandleResult {
    MailboxEmpty,
    Success,
    ActorPanic,
}

#[derive(Clone)]
pub struct ActorRef<A>
where
    A: ActorTrait,
{
    actor: Arc<RwLock<A>>,
    actor_backup: A,
    actor_config: ActorConfig,
    mailbox_in: Sender<MessageEnvelope<A>>,
    mailbox_out: Receiver<MessageEnvelope<A>>,
}

impl<A> ActorRefTrait for ActorRef<A>
where
    A: ActorTrait + Clone + UnwindSafe + 'static,
{
    fn handle(&self) -> HandleResult {
        let mut m = self.mailbox_out.try_recv();
        if m.is_err() {
            return MailboxEmpty
        }
        let mut msg = m.unwrap();
        let mut a = self.actor.write().unwrap();
        let mut ac = a.deref_mut();
        let result = catch_unwind(AssertUnwindSafe(|| msg.handle(ac)));
        //msg.handle(ac);
        match result {
            Err(err) => ActorPanic,
            _ => Success
        }
    }

    fn get_config(&self) -> &ActorConfig {
        &self.actor_config
    }

    fn reset(&mut self) {
        self.actor = Arc::new(RwLock::new(self.actor_backup.clone()));
    }
}

impl<A> ActorRef<A>
where
    A: ActorTrait + Clone,
{
    pub fn new(
        actor: A,
        actor_config: ActorConfig,
        sender: Sender<MessageEnvelope<A>>,
        receiver: Receiver<MessageEnvelope<A>>,
    ) -> Self {
        let actor_backup = actor.clone();
        Self {
            actor: Arc::new(RwLock::new(actor)),
            actor_backup,
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
