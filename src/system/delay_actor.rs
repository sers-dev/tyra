use std::thread::sleep;
use std::time::Duration;
use log::debug;
use crate::message::delayed_message::DelayedMessage;
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, ActorResult, Handler};

pub struct DelayActor {}
impl Actor for DelayActor {}
impl DelayActor {
    fn new() -> Self {
        Self {}
    }
}

pub struct DelayActorFactory {}
impl ActorFactory<DelayActor> for DelayActorFactory {
    fn new_actor(&self, _context: ActorContext<DelayActor>) -> DelayActor {
        DelayActor::new()
    }
}
impl DelayActorFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl<A, M> Handler<DelayedMessage<A, M>> for DelayActor
    where
        M: ActorMessage + 'static,
        A: Actor + Handler<M> + 'static,
{
    fn handle(&mut self, msg: DelayedMessage<A, M>, context: &ActorContext<Self>) -> ActorResult {
        let duration = msg.started.elapsed();
        if duration >= msg.delay {
            let result = msg.destination.send(msg.msg);
            if result.is_err() {
                debug!("");
            }
        }
        else {
            sleep(Duration::from_millis(100));
            let _ = context.actor_ref.send(msg);
        }

        return ActorResult::Ok;
    }
}