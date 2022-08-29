use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_config::{ActorConfig};
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_state::ActorState;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::actor::mailbox::Mailbox;
use crate::message::actor_message::ActorMessage;
use crate::message::envelope::{MessageEnvelope, MessageEnvelopeTrait};
use crate::message::system_stop_message::SystemStopMessage;
use crate::prelude::{Actor, ActorPanicSource, ActorResult};
use crate::system::actor_system::ActorSystem;
use crossbeam_channel::{Receiver, SendTimeoutError};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use crate::system::actor_error::ActorError;

pub trait ExecutorTrait: Send + Sync {
    fn handle(&mut self, is_system_stopping: bool) -> ActorState;
    fn get_config(&self) -> &ActorConfig;
    fn get_address(&self) -> ActorAddress;
    fn is_sleeping(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn wakeup(&mut self);
    fn on_actor_panic(&mut self, source: ActorPanicSource) -> ActorState;
    fn restart_actor(&mut self) -> ActorState;
    fn stop_actor(&mut self, immediately: bool) -> ActorState;
    fn handle_actor_result(&mut self, result: ActorResult) -> ActorState;

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
    A: Actor + 'static,
    P: ActorFactory<A>,
{
}
unsafe impl<A, P> Sync for Executor<A, P>
where
    A: Actor + 'static,
    P: ActorFactory<A>,
{
}

impl<A, P> ExecutorTrait for Executor<A, P>
where
    A: Actor + 'static,
    P: ActorFactory<A>,
{
    fn handle(&mut self, system_is_stopping: bool) -> ActorState {
        if system_is_stopping && !self.system_triggered_stop {
            let result = self.send(SystemStopMessage {});
            if result.is_ok() {
                self.system_triggered_stop = true;
            }
        }
        if self.is_startup {
            self.is_startup = false;
            let result = catch_unwind(AssertUnwindSafe(|| {
                return self.actor.pre_start(&self.context);
            }));
           if result.is_err() {
               return self.on_actor_panic(ActorPanicSource::PreStart);
           }
            else {
                let actor_result = result.unwrap();
                match actor_result {
                    ActorResult::Ok => {}
                    ActorResult::Restart => {
                        return self.restart_actor();
                    }
                    ActorResult::Stop => {
                        return self.stop_actor(false);
                    }
                    ActorResult::Kill => {
                        return self.stop_actor(true);
                    }
                    ActorResult::Sleep(duration) => {
                        return ActorState::Sleeping(duration);
                    }
                }
            }
        }
        let m = self.queue.try_recv();

        if m.is_err() {
            if self.is_stopped() {
                let _ = catch_unwind(AssertUnwindSafe(|| { self.actor.post_stop(&self.context) }));
                return ActorState::Stopped;
            }
            self.mailbox.is_sleeping.store(true, Ordering::Relaxed);
            let duration = self.last_wakeup.elapsed();
            if duration >= Duration::from_millis(5000) {
                return ActorState::Inactive;
            }
            self.mailbox.is_sleeping.store(false, Ordering::Relaxed);
            return ActorState::Running;
        }

        let mut msg = m.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let actor_result = msg.handle(&mut self.actor, &self.context);
            return self.handle_actor_result(actor_result);
        }));
        if result.is_err() {
            return self.on_actor_panic(ActorPanicSource::Message);
        }
        return result.unwrap();
    }

    fn stop_actor(&mut self, immediately: bool) -> ActorState {
        self.mailbox.is_stopped.store(true, Ordering::Relaxed);
        if immediately {
            let _ = catch_unwind(AssertUnwindSafe(|| { self.actor.post_stop(&self.context) }));
            return ActorState::Stopped;
        }
        return ActorState::Running;
    }

    fn restart_actor(&mut self) -> ActorState {
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.actor.pre_restart(&self.context);
            let actor = self.actor_props.new_actor(self.context.clone());
            self.actor = actor;
            self.is_startup = true;
        }));
        if result.is_err() {
            let actor_result = self.actor.on_panic(&self.context, ActorPanicSource::Restart);
            return self.handle_actor_result(actor_result);
        }
        return ActorState::Running;
    }

    fn on_actor_panic(&mut self, source: ActorPanicSource) -> ActorState {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let actor_result = self.actor.on_panic(&self.context, source);
            return self.handle_actor_result(actor_result);
        }));
        if result.is_err() {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let actor_result = self.actor.on_panic(&self.context, ActorPanicSource::OnPanic);
                return self.handle_actor_result(actor_result);
            }));
            if result.is_err() {
                self.stop_actor(true);
            }
            return result.unwrap()
        }
        return result.unwrap();
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

    fn handle_actor_result(&mut self, result: ActorResult) -> ActorState {
        return match result {
            ActorResult::Ok => {
                ActorState::Running
            }
            ActorResult::Restart => {
                self.restart_actor()
            }
            ActorResult::Stop => {
                self.stop_actor(false)
            }
            ActorResult::Kill => {
                self.stop_actor(true)
            }
            ActorResult::Sleep(duration) => {
                ActorState::Sleeping(duration)
            }
        }
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
    ) -> Result<Self, ActorError> {
        let context = ActorContext {
            actor_ref,
            system: system.clone(),
        };

        let actor = catch_unwind(AssertUnwindSafe(|| {
            return actor_props.new_actor(context.clone());
        }));
        if actor.is_err() {
            println!("FUCK");
            return Err(ActorError::InitError);
        }
        return Ok(Self {
            actor: actor.unwrap(),
            actor_props,
            actor_config,
            mailbox,
            queue: receiver,
            actor_address,
            is_startup: true,
            system_triggered_stop: false,
            last_wakeup: Instant::now(),
            context,
        });
    }
    pub fn send<M>(&self, msg: M) -> Result<(), SendTimeoutError<MessageEnvelope<A>>>
    where
        A: Handler<M>,
        M: ActorMessage + 'static,
    {
        return self.mailbox.msg_in.send_timeout(MessageEnvelope::new(msg), Duration::from_millis(10))
    }

}
