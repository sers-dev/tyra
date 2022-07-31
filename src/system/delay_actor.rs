use std::thread::sleep;
use std::time::Duration;
use crate::message::delayed_message::DelayedMessage;
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, Handler};

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
    fn handle(&mut self, msg: DelayedMessage<A, M>, context: &ActorContext<Self>) {
        let duration = msg.started.elapsed();
        if duration >= msg.delay {
            msg.destination.send(msg.msg);
        }
        else {
            sleep(Duration::from_millis(100));
            context.actor_ref.send(msg);
        }

    }
}