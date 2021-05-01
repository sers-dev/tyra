use crate::actor::{ActorAddress, ActorTrait, Handler};
use crate::actor_config::{ActorConfig, RestartPolicy};
use crate::context::Context;
use crate::prelude::TyractorsaurConfig;
use crate::system::ActorSystem;
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use crossbeam_utils::atomic::AtomicCell;
use std::borrow::BorrowMut;
use std::ops::{AddAssign, Deref, DerefMut};
use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use crate::builder::ActorProps;
use crate::message::envelope::MessageEnvelopeTrait;
use crate::message::envelope::MessageEnvelope;
use crate::message::message::MessageTrait;
use crate::message::system_stop_message::SystemStopMessage;
use crate::message::types::MessageType;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::mailbox::Mailbox;


#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActorState {
    Running,
    Sleeping,
    Stopped,
}

pub struct ActorRef<A>
where
    A: ActorTrait + 'static,
{
    mailbox: Mailbox<A>,
    address: ActorAddress,
    system: ActorSystem,
}

impl<A> UnwindSafe for ActorRef<A>
where
    A: ActorTrait + 'static,
{}

impl<A> ActorRef<A>
where
    A: ActorTrait + UnwindSafe,
{
    pub fn new(mailbox: Mailbox<A>, address: ActorAddress, system: ActorSystem) -> Self {
        Self {
            mailbox,
            address,
            system,
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + 'static,
    {
        if self.mailbox.is_stopped() {
            return;
        }

        self.mailbox.send(msg);

        if self.mailbox.is_sleeping() {
            self.system.wakeup(self.address.clone());
        }
    }

    pub fn stop(&self) {
        self.system.remove_actor(&self.address);
        self.send(ActorStopMessage {});
    }

    pub fn get_address(&self) -> ActorAddress {
        self.address.clone()
    }
}

impl<A> Clone for ActorRef<A>
where
    A: ActorTrait + UnwindSafe,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            mailbox: Mailbox {
                is_sleeping: self.mailbox.is_sleeping.clone(),
                is_stopped: self.mailbox.is_stopped.clone(),
                msg_in: self.mailbox.msg_in.clone(),
            },
            address: self.address.clone()
        }
    }
}
