use crate::message::delayed_message::DelayedMessage;
use crate::prelude::{ActorMessage, ActorSystem, ActorWrapper, Handler};
use crate::router::{AddActorMessage, RoundRobinRouter, RoundRobinRouterFactory, RouterMessage};
use crate::system::delay_actor::{DelayActor, DelayActorFactory};
use log::error;
use std::time::Duration;

#[derive(Clone)]
pub struct InternalActorManager {
    delay_router: Option<ActorWrapper<RoundRobinRouter<DelayActor>>>,
}

impl InternalActorManager {
    pub fn new() -> Self {
        Self { delay_router: None }
    }
    pub fn init(&mut self, system: ActorSystem) {
        let delay_builder = system
            .builder()
            .set_pool_name("tyra")
            .set_mailbox_unbounded();
        let delay_router = system
            .builder()
            .set_pool_name("tyra")
            .set_mailbox_unbounded()
            .spawn("delay-router", RoundRobinRouterFactory::new())
            .unwrap();
        let remaining_actors = system.get_available_actor_count_for_pool("tyra").unwrap();
        for i in 0..remaining_actors {
            let delay_actor = delay_builder
                .spawn(format!("delay-{}", i), DelayActorFactory::new())
                .unwrap();
            let result = delay_router.send(AddActorMessage::new(delay_actor));
            if result.is_err() {
                error!("Could not add delay_actor to delay_router");
            }
        }
        self.delay_router = Some(delay_router);
    }

    pub fn send_after<A, M>(&self, msg: M, destination: ActorWrapper<A>, duration: Duration)
    where
        M: ActorMessage + 'static,
        A: Handler<M> + 'static,
    {
        let result =
            self.delay_router
                .as_ref()
                .unwrap()
                .send(RouterMessage::new(DelayedMessage::new(
                    msg,
                    destination,
                    duration,
                )));
        if result.is_err() {
            error!("Could not send message to delay router");
        }
    }
}
