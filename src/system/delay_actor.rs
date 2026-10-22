use crate::message::actor_message::BaseActorMessage;
use crate::message::delayed_message::DelayedMessage;
use crate::prelude::{Actor, ActorContext, ActorFactory, ActorResult, Handler};
use crate as tyra;
use log::error;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

#[derive(Actor, ActorFactory, Clone)]
pub struct DelayActor {}

impl<A, M> Handler<DelayedMessage<A, M>> for DelayActor
where
    M: BaseActorMessage + 'static,
    A: Actor + Handler<M> + 'static,
{
    fn handle(
        &mut self,
        msg: DelayedMessage<A, M>,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        let duration = msg.started.elapsed();
        if duration >= msg.delay {
            let result = msg.destination.send(msg.msg);
            if result.is_err() {
                error!(
                    "Could not delay message to target {}",
                    msg.destination.get_address().actor
                );
            }
        } else {
            sleep(Duration::from_millis(100));
            let _ = context.actor_ref.send(msg);
        }

        return Ok(ActorResult::Ok);
    }
}
