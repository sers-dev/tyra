use crate::actor::actor::Actor;
use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_config::{ActorConfig, RestartPolicy};
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_state::ActorState;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::ActorMessage;
use crate::message::envelope::{MessageEnvelope, MessageEnvelopeTrait};
use crate::message::message_type::MessageType;
use crate::message::system_stop_message::SystemStopMessage;
use crate::system::actor_system::ActorSystem;
use crossbeam_channel::Receiver;
use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

pub trait ExecutorTrait: Send + Sync {
    fn handle(&mut self, is_system_stopping: bool) -> ActorState;
    fn get_config(&self) -> &ActorConfig;
    fn get_address(&self) -> ActorAddress;
    fn is_sleeping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn wakeup(&mut self);
}

pub struct Executor<A, P>
where
    A: Actor + 'static,
    P: ActorFactory<A>,
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
    context: ActorContext<A>,
}

unsafe impl<A, P> Send for Executor<A, P>
where
    A: Actor + UnwindSafe + 'static,
    P: ActorFactory<A>,
{
}
unsafe impl<A, P> Sync for Executor<A, P>
where
    A: Actor + UnwindSafe + 'static,
    P: ActorFactory<A>,
{
}

impl<A, P> ExecutorTrait for Executor<A, P>
where
    A: Actor + UnwindSafe + 'static,
    P: ActorFactory<A>,
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
        let m = self.queue.try_recv();

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

impl<A, P> Executor<A, P>
where
    A: Actor,
    P: ActorFactory<A>,
{
    pub fn new(
        actor_props: P,
        actor_address: ActorAddress,
        actor_config: ActorConfig,
        mailbox: Mailbox<A>,
        receiver: Receiver<MessageEnvelope<A>>,
        system: ActorSystem,
        actor_ref: ActorWrapper<A>,
    ) -> Self {

        let context = ActorContext {
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
            context,
        }
    }
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: ActorMessage + 'static,
    {
        self.mailbox.msg_in.send(MessageEnvelope::new(msg)).unwrap();
    }
}
