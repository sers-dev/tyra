use crate::actor::actor::Actor;
use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_send_error::ActorSendError;
use crate::actor::handler::Handler;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::BaseActorMessage;
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::system_state::SystemState;
use crate::system::wakeup_manager::WakeupManager;
use crate::wrapper::local_actor_wrapper::LocalActorWrapper;
use crate::wrapper::remote_actor_wrapper::RemoteActorWrapper;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::UnwindSafe;
use std::time::Duration;
use crate::prelude::ActorSystem;

#[derive(Serialize, Deserialize)]
#[serde(bound(
serialize = "A: Actor",
deserialize = "A: Actor",
))]
pub struct ActorWrapper<A>
where
    A: Actor,
{
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
    A: Actor + UnwindSafe + 'static,
{
    /// Automatically called by the [ActorBuilder.spawn](../prelude/struct.ActorBuilder.html#method.spawn)
    pub fn new(
        mailbox: Mailbox<A>,
        address: ActorAddress,
        wakeup_manager: WakeupManager,
        internal_actor_manager: InternalActorManager,
        system_state: SystemState,
    ) -> Self {
        let local = Some(LocalActorWrapper::new(
            mailbox,
            wakeup_manager,
            internal_actor_manager,
        ));
        let remote = RemoteActorWrapper::new(system_state);
        Self {
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
        if self.local.is_some() {
            return self.local.as_ref().unwrap().send(msg, self.address.clone());
        }
        return self.remote.send(msg, &self.address);
    }

    /// Same as send, but with a user defined timeout
    pub fn send_timeout<M>(&self, msg: M, timeout: Duration) -> Result<(), ActorSendError>
    where
        A: Handler<M>,
        M: BaseActorMessage + 'static,
    {
        if self.local.is_some() {
            return self
                .local
                .as_ref()
                .unwrap()
                .send_timeout(msg, timeout, self.address.clone());
        }
        return self.remote.send_timeout(msg, timeout, self.address.clone());
    }

    /// Sends a message to the actor after a specified delay
    pub fn send_after<M>(&self, msg: M, delay: Duration) -> Result<(), ActorSendError>
    where
        A: Handler<M> + 'static,
        M: BaseActorMessage + 'static,
    {
        if self.local.is_some() {
            return self
                .local
                .as_ref()
                .unwrap()
                .send_after(msg, delay, self.clone());
        }
        return self.remote.send_after(msg, delay, self.address.clone());
    }

    /// Tells the actor to stop accepting message and to shutdown after all existing messages have been processed
    pub fn stop(&self) -> Result<(), ActorSendError> {
        if self.local.is_some() {
            return self.local.as_ref().unwrap().stop(self.address.clone());
        }
        return self.remote.stop();
    }

    /// Tells the actor to sleep for the specified duration
    pub fn sleep(&self, duration: Duration) -> Result<(), ActorSendError> {
        if self.local.is_some() {
            return self
                .local
                .as_ref()
                .unwrap()
                .sleep(duration, self.address.clone());
        }
        return self.remote.sleep(duration, self.address.clone());
    }

    /// Returns a reference to the address of the actor
    /// Returns a reference to the address of the actor
    pub fn get_address(&self) -> &ActorAddress {
        &self.address
    }

    /// Returns the current mailbox size
    pub fn get_mailbox_size(&self) -> usize {
        if self.local.is_some() {
            return self.local.as_ref().unwrap().get_mailbox_size();
        }
        return self.remote.get_mailbox_size();
    }

    /// Returns true if an actor is no longer accepting messages
    pub fn is_mailbox_stopped(&self) -> bool {
        if self.local.is_some() {
            return self.local.as_ref().unwrap().is_mailbox_stopped();
        }
        return self.remote.is_mailbox_stopped();
    }

    /// Returns true if an actor has been completely stopped after processing all messages that are still within the queue
    pub fn is_stopped(&self) -> bool {
        if self.local.is_some() {
            return self.local.as_ref().unwrap().is_stopped();
        }
        return self.remote.is_stopped();
    }

    /// Blocks until the actor has been stopped
    pub fn wait_for_stop(&self) {
        if self.local.is_some() {
            return self
                .local
                .as_ref()
                .unwrap()
                .wait_for_stop(self.address.clone());
        }
        return self.remote.wait_for_stop();
    }

    /// This function is required after deserializing any ActorWrapper<A>
    /// If the actor is a local actor it will restore the LocalActorWrapper and any message can be sent directly to the actor again
    /// If the actor is a remote actor it will add the SystemState to it, so that the messages can be forwarded to the destination
    /// It is technically impossible to deserialize a working LocalActor<A> or a RemoteActor directly, which is why this helper is required to feed the unserializable information back into it
    pub fn init_after_deserialize(&mut self, system: &ActorSystem) {
        let actor_wrapper = system.builder::<A>().init_after_deserialize(self.get_address());
        self.local = actor_wrapper.local;
        self.remote = actor_wrapper.remote;
    }

    /// Returns an ActorWrapper<A> that will be handled as a remote actor.
    /// If the actor exists locally tru `init_after_deserialize` or `ActorBuilder<A>::get_existing()` instead
    pub fn from_address(address: ActorAddress, system_state: SystemState) -> Self {
        return Self {
            address,
            remote: RemoteActorWrapper::new(system_state),
            local: None,
        }
    }

}

impl<A> Clone for ActorWrapper<A>
where
    A: Actor + UnwindSafe,
{
    fn clone(&self) -> Self {
        Self {
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
