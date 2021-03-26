use crate::actor::{ActorTrait, Handler, ActorAddress};
use crate::actor_config::{ActorConfig, RestartPolicy};
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait};
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

pub trait ActorRefTrait: Send + Sync {
    fn handle(&mut self) -> ActorState;
    fn get_config(&self) -> &ActorConfig;
    fn get_address(&self) -> ActorAddress;
    fn is_sleeping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn get_mailbix_size(&self) -> usize;
    fn wakeup(&mut self);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActorState {
    Running,
    Sleeping,
    Stopped,
}

#[derive(Clone)]
pub struct ActorHandler<A>
where
    A: ActorTrait,
{
    actor: A,
    actor_backup: A,
    actor_config: ActorConfig,
    mailbox_in: Sender<MessageEnvelope<A>>,
    mailbox_out: Receiver<MessageEnvelope<A>>,
    actor_address: ActorAddress,
    is_stopped: Arc<AtomicBool>,
    is_sleeping: Arc<AtomicBool>,
    last_wakeup: Instant,
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

        let mut m = self.mailbox_out.try_recv();

        if m.is_err() {
            let duration = self.last_wakeup.elapsed();
            if duration >= Duration::from_secs(1) {
                self.is_sleeping.store(true, Ordering::Relaxed);
                return ActorState::Sleeping;
            }
            return ActorState::Running;
        }

        let mut msg = m.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| msg.handle(&mut self.actor)));
        if result.is_err() {
            println!("ACTOR PANIC");
            if self.actor_config.restart_policy == RestartPolicy::Never {
                self.is_stopped.store(true, Ordering::Relaxed);
                return ActorState::Stopped
            }
            self.actor = self.actor_backup.clone();
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
        self.is_sleeping.load(Ordering::Relaxed)
    }

    fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }

    fn get_mailbix_size(&self) -> usize {
        self.mailbox_out.len()
    }

    fn wakeup(&mut self) {
        self.is_sleeping.store(false, Ordering::Relaxed);
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
        sender: Sender<MessageEnvelope<A>>,
        receiver: Receiver<MessageEnvelope<A>>,
    ) -> Self {
        let actor_backup = actor.clone();
        let actor_address = ActorAddress{
            actor: actor_config.actor_name.clone(),
            system: String::from("default"),
            pool: actor_config.pool_name.clone(),
            remote: String::from("local"),
        };

        Self {
            actor,
            actor_backup,
            actor_config,
            mailbox_in: sender,
            mailbox_out: receiver,
            actor_address,
            is_stopped: Arc::new(AtomicBool::new(false)),
            is_sleeping: Arc::new(AtomicBool::new(true)),
            last_wakeup: Instant::now(),
        }
    }
    pub fn send<M>(& self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
    {
            self.mailbox_in.send(MessageEnvelope::new(msg)).unwrap();
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
    actor_ref: ActorHandler<A>,
    system: ActorSystem,
}


impl<A> ActorRef<A>
    where
        A: ActorTrait + Clone + UnwindSafe + ,
{
    pub fn new(
        actor_ref: ActorHandler<A>,
        system: ActorSystem
    ) -> Self {
        Self {
            actor_ref,
            system
        }
    }

    pub fn send<M>(& self, msg: M)
    where
        A: Handler<M>,
        M: MessageTrait + Clone + 'static,
    {

        if self.actor_ref.is_stopped() {
            return;
        }

        self.actor_ref.send(msg);

        if self.actor_ref.is_sleeping() {
            self.system.wakeup(self.actor_ref.get_address());
        }
    }
    pub fn get_config(&self) -> ActorConfig {
        self.actor_ref.get_config().clone()
    }
}