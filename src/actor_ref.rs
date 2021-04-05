use crate::actor::{ActorTrait, Handler, ActorAddress};
use crate::actor_config::{ActorConfig, RestartPolicy};
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait, StopMessage, MessageType};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::{DerefMut, Deref, AddAssign};
use std::sync::{Arc, RwLock};
use std::panic::{UnwindSafe, AssertUnwindSafe, catch_unwind};
use crossbeam_utils::atomic::AtomicCell;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::system::ActorSystem;
use std::time::{Duration, Instant};
use crate::context::Context;
use crate::prelude::TyractorsaurConfig;

pub trait ActorRefTrait: Send + Sync {
    fn handle(&mut self) -> ActorState;
    fn get_config(&self) -> &ActorConfig;
    fn get_address(&self) -> ActorAddress;
    fn is_sleeping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn wakeup(&mut self);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActorState {
    Running,
    Sleeping,
    Stopped,
}

#[derive(Clone)]
pub struct Mailbox<A> {
    pub is_stopped: Arc<AtomicBool>,
    pub is_sleeping: Arc<AtomicBool>,
    pub msg_in: Sender<MessageEnvelope<A>>,
}

impl<A> Mailbox<A>
    where
        A: ActorTrait + Clone,
{
    pub fn send<M>(& self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait + Clone + 'static,
    {
        self.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }

    fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }

    fn wakeup(&mut self) {
        self.is_sleeping.store(false, Ordering::Relaxed);
    }

}

/////////////



pub struct ActorHandler<A>
where
    A: ActorTrait + 'static,
{
    actor: A,
    actor_backup: A,
    actor_config: ActorConfig,
    mailbox: Mailbox<A>,
    queue: Receiver<MessageEnvelope<A>>,
    actor_address: ActorAddress,
    is_startup: bool,
    last_wakeup: Instant,
    system: ActorSystem,
    context: Context<A>,
}

unsafe impl<A> Send for ActorHandler<A>
where
    A: ActorTrait + Clone + UnwindSafe + 'static
{}
unsafe impl<A> Sync for ActorHandler<A>
where
    A: ActorTrait + Clone + UnwindSafe + 'static
{}

impl<A> ActorRefTrait for ActorHandler<A>
where
    A: ActorTrait + Clone + UnwindSafe + 'static,
{
    fn handle(&mut self) -> ActorState {

        if self.is_startup {
            self.is_startup = false;
            self.actor.pre_start();
        }
        let mut m = self.queue.try_recv();

        if m.is_err() {
            if self.is_stopped() {
                self.actor.post_stop();
                return ActorState::Stopped
            }
            let duration = self.last_wakeup.elapsed();
            if duration >= Duration::from_secs(1) {
                self.mailbox.is_sleeping.store(true, Ordering::Relaxed);
                return ActorState::Sleeping;
            }
            return ActorState::Running;
        }

        let mut msg = m.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| msg.handle(&mut self.actor, &self.context)));
        if result.is_err() {
            println!("ACTOR PANIC");
            self.actor.post_stop();

            if self.actor_config.restart_policy == RestartPolicy::Never || self.is_stopped() {
                self.mailbox.is_stopped.store(true, Ordering::Relaxed);
                return ActorState::Stopped
            }
            self.actor = self.actor_backup.clone();
            self.is_startup = true;
            return ActorState::Running
        }
        let message_type = result.unwrap();
        if message_type == MessageType::StopMessage {
            self.mailbox.is_stopped.store(true, Ordering::Relaxed);
            return ActorState::Running
        }

        ActorState::Running
    }

    fn get_config(&self) -> &ActorConfig {
        &self.actor_config
    }

    fn get_address(&self) -> ActorAddress {
        self.actor_address.clone()
    }

    fn is_sleeping(&self) -> bool {
        self.mailbox.is_sleeping.load(Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.mailbox.is_stopped.load(Ordering::Relaxed)
    }

    fn wakeup(&mut self) {
        self.mailbox.is_sleeping.store(false, Ordering::Relaxed);
        self.last_wakeup = Instant::now();
    }
}

impl<A> ActorHandler<A>
where
    A: ActorTrait + Clone,
{
    pub fn new(
        actor: A,
        actor_config: ActorConfig,
        mailbox: Mailbox<A>,
        receiver: Receiver<MessageEnvelope<A>>,
        system: ActorSystem,
        system_name: String,
        actor_ref: ActorRef<A>,
    ) -> Self {
        let actor_backup = actor.clone();
        let actor_address = ActorAddress{
            actor: actor_config.actor_name.clone(),
            system: system_name,
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };

        let context = Context {
            actor_ref,
            system: system.clone(),
        };

        Self {
            actor,
            actor_backup,
            actor_config,
            mailbox,
            queue: receiver,
            actor_address,
            is_startup: true,
            last_wakeup: Instant::now(),
            system,
            context
        }
    }
    pub fn send<M>(& self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
    {
            self.mailbox.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }

}


pub trait ActorRefOuterTrait: Send + Sync {
    fn get_config(&self) -> ActorConfig;
}

#[derive(Clone)]
pub struct ActorRef<A>
    where
        A: ActorTrait + 'static,
{
    mailbox: Mailbox<A>,
    address: ActorAddress,
    system: ActorSystem,
}


impl<A> ActorRef<A>
    where
        A: ActorTrait + Clone + UnwindSafe + ,
{
    pub fn new(
        mailbox: Mailbox<A>,
        address: ActorAddress,
        system: ActorSystem,
    ) -> Self {
        Self {
            mailbox,
            address,
            system,
        }
    }

    pub fn send<M>(& self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
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
        self.send(StopMessage {});
    }
}