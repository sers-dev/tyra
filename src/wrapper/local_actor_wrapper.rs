use std::panic::UnwindSafe;
use std::thread::sleep;
use std::time::Duration;
use crate::actor::actor_address::ActorAddress;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::BaseActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::sleep_message::SleepMessage;
use crate::actor::actor_send_error::ActorSendError;
use crate::actor::actor::Actor;
use crate::actor::handler::Handler;
use crate::prelude::ActorWrapper;
use crate::system::internal_actor_manager::InternalActorManager;
use crate::system::wakeup_manager::WakeupManager;

pub struct LocalActorWrapper<A>
    where
        A: Actor,
{
    mailbox: Mailbox<A>,
    wakeup_manager: WakeupManager,
    internal_actor_manager: Box<InternalActorManager>,
}


impl<A> LocalActorWrapper<A>
    where
        A: Actor + UnwindSafe,
{
    pub fn new(
        mailbox: Mailbox<A>,
        wakeup_manager: WakeupManager,
        internal_actor_manager: InternalActorManager,
    ) -> Self {
        Self {
            mailbox,
            wakeup_manager,
            internal_actor_manager: Box::new(internal_actor_manager),
        }
    }

    pub fn send<M>(&self, msg: M, address: ActorAddress) -> Result<(), ActorSendError>
        where
            A: Handler<M>,
            M: BaseActorMessage + 'static,
    {
        if self.mailbox.is_stopped() {
            return Err(ActorSendError::AlreadyStoppedError);
        }

        let result = self.mailbox.send(msg);

        if result.is_err() {
            return result;
        }

        if self.mailbox.is_sleeping() {
            self.wakeup_manager.wakeup(address);
        }

        return Ok(());
    }

    pub fn send_timeout<M>(&self, msg: M, timeout: Duration, address: ActorAddress) -> Result<(), ActorSendError>
        where
            A: Handler<M>,
            M: BaseActorMessage + 'static,
    {
        if self.mailbox.is_stopped() {
            return Err(ActorSendError::AlreadyStoppedError);
        }

        let result = self.mailbox.send_timeout(msg, timeout);

        if result.is_err() {
            return result;
        }

        if self.mailbox.is_sleeping() {
            self.wakeup_manager.wakeup(address);
        }

        return Ok(());
    }

    pub fn send_after<M>(&self, msg: M, delay: Duration, destination: ActorWrapper<A>) -> Result<(), ActorSendError>
        where
            A: Handler<M> + 'static,
            M: BaseActorMessage + 'static,
    {
        if self.mailbox.is_stopped() {
            return Err(ActorSendError::AlreadyStoppedError);
        }

        self.internal_actor_manager
            .send_after(msg, destination, delay);

        return Ok(());
    }

    pub fn stop(&self, address: ActorAddress) -> Result<(), ActorSendError> {
        return self.send(ActorStopMessage::new(), address);
    }

    pub fn sleep(&self, duration: Duration, address: ActorAddress) -> Result<(), ActorSendError> {
        return self.send(SleepMessage { duration }, address);
    }

    pub fn get_mailbox_size(&self) -> usize {
        return self.mailbox.len();
    }

    pub fn is_mailbox_stopped(&self) -> bool {
        return self.mailbox.is_stopped()
    }

    pub fn is_stopped(&self) -> bool {
        return self.get_mailbox_size() == 0 && self.mailbox.is_stopped()
    }

    pub fn wait_for_stop(&self, address: ActorAddress) {
        let _ = self.stop(address);
        while !self.is_stopped() {
            sleep(Duration::from_millis(25));
        }
    }
}

impl<A> Clone for LocalActorWrapper<A>
    where
        A: Actor + UnwindSafe,
{
    fn clone(&self) -> Self {
        Self {
            wakeup_manager: self.wakeup_manager.clone(),
            mailbox: self.mailbox.clone(),
            internal_actor_manager: self.internal_actor_manager.clone(),
        }
    }
}