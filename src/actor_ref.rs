use crate::actor::{ActorTrait, Handler};
use crate::actor_config::{ActorConfig, RestartPolicy};
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::panic::{UnwindSafe, AssertUnwindSafe, catch_unwind};

pub trait ActorRefTrait: Send + Sync {
    fn handle(&mut self);
    fn get_config(&self) -> &ActorConfig;
    fn handle_panic(&mut self);
    fn get_current_state(&self) -> ActorState;
}

#[derive(PartialEq, Clone)]
pub enum ActorState {
    Running,
    Sleeping,
    Stopped,
}

#[derive(Clone)]
pub struct ActorRef<A>
where
    A: ActorTrait,
{
    actor: A,
    actor_backup: A,
    actor_config: ActorConfig,
    mailbox_in: Sender<MessageEnvelope<A>>,
    mailbox_out: Receiver<MessageEnvelope<A>>,
    actor_state: ActorState
}

impl<A> ActorRefTrait for ActorRef<A>
where
    A: ActorTrait + Clone + UnwindSafe + 'static,
{
    fn handle(&mut self) {
        let mut m = self.mailbox_out.try_recv();
        if m.is_err() {
            self.actor_state = ActorState::Sleeping;
            return;
        }
        let mut msg = m.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| msg.handle(&mut self.actor)));
        if result.is_err() {
            self.handle_panic();
        }
    }

    fn get_config(&self) -> &ActorConfig {
        &self.actor_config
    }

    fn handle_panic(&mut self) {
        if self.actor_config.restart_policy == RestartPolicy::Never {
            self.actor_state = ActorState::Stopped;
            return;
        }
        self.actor = self.actor_backup.clone();
    }

    fn get_current_state(&self) -> ActorState {
        self.actor_state.clone()
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
            actor,
            actor_backup,
            actor_config,
            actor_state: ActorState::Running,
            mailbox_in: sender,
            mailbox_out: receiver,
        }
    }
    pub fn send<M>(&mut self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
    {
        if self.actor_state != ActorState::Stopped {
            self.mailbox_in.send(MessageEnvelope::new(msg));
        }
    }
}
