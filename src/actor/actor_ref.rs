use crate::actor::actor::{ActorTrait, ActorAddress, Handler};
use crate::actor::mailbox::Mailbox;
use crate::system::ActorSystem;
use std::panic::UnwindSafe;
use crate::message::message::MessageTrait;
use crate::message::actor_stop_message::ActorStopMessage;

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
