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
use crate::message::sleep_message::SleepMessage;
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

    /// Sends a message to the actor that is then processed through the corresponding Handler<M> implementation
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

    /// Sends a message to the actor after a specified delay
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

    /// Tells the actor to stop accepting message and to shutdown after all existing messages have been processed
    pub fn stop(&self) {
        self.send(ActorStopMessage {});
    }

    /// Tells the actor to sleep for the specified duration
    pub fn sleep(&self, duration: Duration) {
        self.send(SleepMessage{
            duration
        });
    }

    /// Returns a reference to the address of the actor
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
