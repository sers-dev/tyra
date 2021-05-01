use crate::actor::actor_state::ActorState;
use crate::actor::actor_config::{ActorConfig, RestartPolicy};
use crate::actor::actor::{ActorAddress, ActorTrait, Handler};
use crate::actor::builder::ActorProps;
use crate::actor::mailbox::Mailbox;
use crossbeam_channel::Receiver;
use crate::message::envelope::{MessageEnvelope, MessageEnvelopeTrait};
use std::time::{Instant, Duration};
use crate::system::ActorSystem;
use crate::actor::context::Context;
use std::panic::{UnwindSafe, catch_unwind, AssertUnwindSafe};
use crate::message::system_stop_message::SystemStopMessage;
use std::sync::atomic::Ordering;
use crate::message::types::MessageType;
use crate::actor::actor_ref::ActorRef;
use crate::message::message::MessageTrait;

pub trait ActorHandlerTrait: Send + Sync {
    fn handle(&mut self, system_is_stopping: bool) -> ActorState;
    fn get_config(&self) -> &ActorConfig;
    fn get_address(&self) -> ActorAddress;
    fn is_sleeping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn wakeup(&mut self);
}

pub struct ActorHandler<A, P>
    where
        A: ActorTrait + 'static,
        P: ActorProps<A>
{
    actor: A,
    actor_props: P,
    actor_config: ActorConfig,
    mailbox: Mailbox<A>,
    queue: Receiver<MessageEnvelope<A>>,
    actor_address: ActorAddress,
    is_startup: bool,
    system_triggered_stop: bool,
    last_wakeup: Instant,
    system: ActorSystem,
    context: Context<A>,
}

unsafe impl<A, P> Send for ActorHandler<A, P> where A: ActorTrait + UnwindSafe + 'static, P: ActorProps<A> {}
unsafe impl<A, P> Sync for ActorHandler<A, P> where A: ActorTrait + UnwindSafe + 'static, P: ActorProps<A> {}

impl<A, P> ActorHandlerTrait for ActorHandler<A, P>
    where
        A: ActorTrait + UnwindSafe + 'static,
        P: ActorProps<A>
{
    fn handle(&mut self, system_is_stopping: bool) -> ActorState {
        if system_is_stopping && !self.system_triggered_stop {
            self.system_triggered_stop = true;
            self.send(SystemStopMessage {});
        }
        if self.is_startup {
            self.is_startup = false;
            self.actor.pre_start();
        }
        let mut m = self.queue.try_recv();

        if m.is_err() {
            if self.is_stopped() {
                self.actor.post_stop();
                return ActorState::Stopped;
            }
            self.mailbox.is_sleeping.store(true, Ordering::Relaxed);
            let duration = self.last_wakeup.elapsed();
            if duration >= Duration::from_secs(5) {
                return ActorState::Sleeping;
            }
            self.mailbox.is_sleeping.store(false, Ordering::Relaxed);
            return ActorState::Running;
        }

        let mut msg = m.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            msg.handle(&mut self.actor, &self.context)
        }));
        if result.is_err() {
            println!("ACTOR PANIC");
            self.actor.post_stop();

            if self.actor_config.restart_policy == RestartPolicy::Never || self.is_stopped() {
                self.mailbox.is_stopped.store(true, Ordering::Relaxed);
                return ActorState::Stopped;
            }
            self.actor = self.actor_props.new_actor(self.context.clone());
            self.is_startup = true;
            return ActorState::Running;
        }
        let message_type = result.unwrap();
        if message_type == MessageType::ActorStopMessage {
            self.mailbox.is_stopped.store(true, Ordering::Relaxed);
            return ActorState::Running;
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

impl<A, P> ActorHandler<A, P>
    where
        A: ActorTrait,
        P: ActorProps<A>
{
    pub fn new(
        actor_props: P,
        actor_config: ActorConfig,
        mailbox: Mailbox<A>,
        receiver: Receiver<MessageEnvelope<A>>,
        system: ActorSystem,
        system_name: String,
        actor_ref: ActorRef<A>,
    ) -> Self {
        let actor_address = ActorAddress {
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
            actor: actor_props.new_actor(context.clone()),
            actor_props,
            actor_config,
            mailbox,
            queue: receiver,
            actor_address,
            is_startup: true,
            system_triggered_stop: false,
            last_wakeup: Instant::now(),
            system,
            context,
        }
    }
    pub fn send<M>(&self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait + 'static,
    {
        self.mailbox.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }
}