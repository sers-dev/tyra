use std::fmt::{Debug, Formatter};
use crate::actor::actor_address::ActorAddress;
use crate::actor::handler::Handler;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::prelude::Actor;
use crate::system::wakeup_manager::WakeupManager;
use std::panic::UnwindSafe;
use std::time::Duration;
use crate::system::internal_actor_manager::InternalActorManager;

/// Wrapper used to interact with [Actor]
pub struct ActorWrapper<A>
where
    A: Actor,
{
    mailbox: Mailbox<A>,
    address: ActorAddress,
    wakeup_manager: WakeupManager,
    internal_actor_manager: Box<InternalActorManager>,
}

impl<A> Debug for ActorWrapper<A> where A: Actor{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl<A> UnwindSafe for ActorWrapper<A> where A: Actor {}

impl<A> ActorWrapper<A>
where
    A: Actor + UnwindSafe,
{
    /// Automatically called by the [ActorBuilder.build](../prelude/struct.ActorBuilder.html#method.build)
    pub fn new(mailbox: Mailbox<A>, address: ActorAddress, wakeup_manager: WakeupManager, internal_actor_manager: InternalActorManager) -> Self {
        Self {
            mailbox,
            address,
            wakeup_manager,
            internal_actor_manager: Box::new(internal_actor_manager)
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: ActorMessage + 'static,
    {
        if self.mailbox.is_stopped() {
            return;
        }

        self.mailbox.send(msg);

        if self.mailbox.is_sleeping() {
            self.wakeup_manager.wakeup(self.address.clone());
        }
    }

    pub fn send_after<M>(&self, msg: M, delay: Duration)
        where
            A: Handler<M> + 'static,
            M: ActorMessage + 'static,
    {
        if self.mailbox.is_stopped() {
            return;
        }

        self.internal_actor_manager.send_after(msg, self.clone(), delay);

    }

    pub fn stop(&self) {
        self.send(ActorStopMessage {});
    }

    pub fn get_address(&self) -> &ActorAddress {
        &self.address
    }
}

impl<A> Clone for ActorWrapper<A>
where
    A: Actor + UnwindSafe,
{
    fn clone(&self) -> Self {
        Self {
            wakeup_manager: self.wakeup_manager.clone(),
            mailbox: self.mailbox.clone(),
            address: self.address.clone(),
            internal_actor_manager: self.internal_actor_manager.clone(),
        }
    }
}
