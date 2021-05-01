use crate::actor::actor::Actor;
use crate::actor::mailbox::Mailbox;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::actor::actor_address::ActorAddress;
use crate::actor::handler::Handler;

pub struct ActorWrapper<A>
where
    A: Actor + 'static,
{
    mailbox: Mailbox<A>,
    address: ActorAddress,
    system: ActorSystem,
}

impl<A> UnwindSafe for ActorWrapper<A>
where
    A: Actor + 'static,
{}

impl<A> ActorWrapper<A>
where
    A: Actor + UnwindSafe,
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
        M: ActorMessage + 'static,
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

impl<A> Clone for ActorWrapper<A>
where
    A: Actor + UnwindSafe,
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
