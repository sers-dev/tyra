use crate::actor::{ActorTrait, Handler, ActorAddress};
use crate::actor_config::{ActorConfig, RestartPolicy};
use crate::message::{MessageEnvelope, MessageEnvelopeTrait, MessageTrait};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::{DerefMut, Deref};
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
    last_active: Instant,
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
            //let duration = self.last_active.elapsed();
            //if duration >= Duration::from_secs(1) {
                //println!("SLEEPI BOY3");
                return ActorState::Sleeping;
            //}
            return ActorState::Running;
        }
        //self.last_active = Instant::now();

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
        self.mailbox_out.len() == 0
        //self.last_active.load().elapsed() >= Duration::from_secs(1)
    }

    fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }

    fn get_mailbix_size(&self) -> usize {
        self.mailbox_out.len()
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
            last_active: Instant::now(),
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

        let is_sleeping = self.actor_ref.is_sleeping();

        self.actor_ref.send(msg);

        //if is_sleeping {
        //   self.system.wakeup(self.actor_ref.get_address());
        //}

    }
    pub fn get_config(&self) -> ActorConfig {
        self.actor_ref.get_config().clone()
    }
}