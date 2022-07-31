use std::time::Duration;
use crate::message::delayed_message::DelayedMessage;
use crate::prelude::{ActorMessage, ActorSystem, ActorWrapper, Handler};
use crate::router::{AddActorMessage, RoundRobinRouter, RoundRobinRouterFactory, RouterMessage};
use crate::system::delay_actor::{DelayActor, DelayActorFactory};

#[derive(Clone)]
pub struct InternalActorManager {
    delay_router: Option<ActorWrapper<RoundRobinRouter<DelayActor>>>,
}

impl InternalActorManager {
    pub fn new() -> Self {
        Self {
            delay_router: None
        }
    }
    pub fn init(&mut self, system: ActorSystem) {
        let delay_builder = system.builder().set_pool_name("tyra").set_mailbox_unbounded();
        let delay_router = system.builder().set_pool_name("tyra").set_mailbox_unbounded().spawn("delay-router", RoundRobinRouterFactory::new()).unwrap();
        for i in 0..3 {
            let delay_actor = delay_builder.spawn(format!("delay-{}", i), DelayActorFactory::new()).unwrap();
            delay_router.send(AddActorMessage::new(delay_actor));
        }
        //let delay_actor = system.builder().set_pool_name("tyra").set_mailbox_unbounded().spawn("delay", DelayActorFactory::new()).unwrap();
        self.delay_router = Some(delay_router);
    }

    pub fn send_after<A, M>(&self, msg: M, destination: ActorWrapper<A>, duration: Duration)
    where
        M: ActorMessage + 'static,
        A: Handler<M> + 'static
    {
        self.delay_router.as_ref().unwrap().send(RouterMessage::new(DelayedMessage::new(msg, destination, duration)));
    }

}