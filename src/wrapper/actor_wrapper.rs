use std::fmt::{Debug, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::UnwindSafe;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::actor::actor_address::ActorAddress;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::BaseActorMessage;
use crate::message::sleep_message::SleepMessage;
use crate::actor::actor_send_error::ActorSendError;
use crate::actor::actor::Actor;
use crate::actor::handler::Handler;
use crate::prelude::ActorSystem;
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::system_state::SystemState;
use crate::system::wakeup_manager::WakeupManager;
use crate::wrapper::local_actor_wrapper::LocalActorWrapper;
use crate::wrapper::remote_actor_wrapper::RemoteActorWrapper;

#[derive(Serialize, Deserialize)]
pub struct ActorWrapper<A>
    where
        A: Actor,
{
    is_local: bool,
    address: ActorAddress,
    remote: RemoteActorWrapper,
    #[serde(skip)]
    local: Option<LocalActorWrapper<A>>,

}

impl<A> Debug for ActorWrapper<A>
    where
        A: Actor,
{
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
    pub fn new(
        mailbox: Mailbox<A>,
        address: ActorAddress,
        wakeup_manager: WakeupManager,
        internal_actor_manager: InternalActorManager,
        system_state: SystemState,
    ) -> Self {
        let is_local = true;
        let local = Some(LocalActorWrapper::new(mailbox, wakeup_manager, internal_actor_manager));
        let remote = RemoteActorWrapper::new(system_state);
        Self {
            is_local,
            address,
            remote,
            local,
        }
    }

    /// Sends a message to the actor that is then processed through the corresponding Handler<M> implementation
    /// Blocks until message has been sent, or fails if the target has been stopped
    /// It is NOT recommended to use this to send messages to Actors with a limited mailbox. Use send_timeout() or send_after() for these cases
    pub fn send<M>(&self, msg: M) -> Result<(), ActorSendError>
        where
            A: Handler<M>,
            M: BaseActorMessage + 'static,
    {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().send(msg, self.address.clone());
        }
        else {
            return self.remote.send(msg, &self.address);
        }
    }

    /// Same as send, but with a user defined timeout
    pub fn send_timeout<M>(&self, msg: M, timeout: Duration) -> Result<(), ActorSendError>
        where
            A: Handler<M>,
            M: BaseActorMessage + 'static,
    {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().send_timeout(msg, timeout, self.address.clone());
        }
        else {
            return self.remote.send_timeout(msg, timeout, self.address.clone());
        }
    }

    /// Sends a message to the actor after a specified delay
    pub fn send_after<M>(&self, msg: M, delay: Duration) -> Result<(), ActorSendError>
        where
            A: Handler<M> + 'static,
            M: BaseActorMessage + 'static,
    {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().send_after(msg, delay, self.clone());
        }
        else {
            return self.remote.send_after(msg, delay, self.address.clone());
        }
    }

    /// Tells the actor to stop accepting message and to shutdown after all existing messages have been processed
    pub fn stop(&self) -> Result<(), ActorSendError> {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().stop(self.address.clone());
        }
        else {
            return self.remote.stop();
        }
    }

    /// Tells the actor to sleep for the specified duration
    pub fn sleep(&self, duration: Duration) -> Result<(), ActorSendError> {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().send(SleepMessage{ duration}, self.address.clone());
        }
        else {
            return self.remote.send(SleepMessage{ duration}, &self.address);
        }
    }

    /// Returns a reference to the address of the actor
    /// Returns a reference to the address of the actor
    pub fn get_address(&self) -> &ActorAddress {
        &self.address
    }

    /// Returns the current mailbox size
    pub fn get_mailbox_size(&self) -> usize {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().get_mailbox_size();
        }
        else {
            return self.remote.get_mailbox_size();
        }
    }

    /// Returns true if an actor is no longer accepting messages
    pub fn is_mailbox_stopped(&self) -> bool {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().is_mailbox_stopped();
        }
        else {
            return self.remote.is_mailbox_stopped();
        }
    }

    /// Returns true if an actor has been completely stopped after processing all messages that are still within the queue
    pub fn is_stopped(&self) -> bool {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().is_stopped();
        }
        else {
            return self.remote.is_stopped();
        }
    }

    /// Blocks until the actor has been stopped
    pub fn wait_for_stop(&self) {
        if self.is_local && self.local.is_some() {
            return self.local.as_ref().unwrap().wait_for_stop(self.address.clone());
        }
        else {
            return self.remote.wait_for_stop();
        }
    }
}

impl<A> Clone for ActorWrapper<A>
    where
        A: Actor + UnwindSafe,
{
    fn clone(&self) -> Self {
        Self {
            is_local: self.is_local.clone(),
            remote: self.remote.clone(),
            local: self.local.clone(),
            address: self.address.clone(),
        }
    }
}

impl<A> Hash for ActorWrapper<A>
    where
        A: Actor + UnwindSafe,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}