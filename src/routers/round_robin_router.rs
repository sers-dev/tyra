use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::{ActorMessage, BaseActorMessage};
use crate::prelude::{Actor, ActorFactory, ActorResult, BulkActorMessage};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::bulk_router_message::BulkRouterMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate as tyra;
use log::error;
use std::error::Error;

#[derive(Actor, ActorFactory)]
pub struct RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    #[tyra(default = "0")]
    route_index: usize,
    #[tyra(default = "Vec::new()")]
    route_to: Vec<ActorWrapper<A>>,
    #[tyra(default = "false")]
    can_route: bool,
}

impl<A> RoundRobinRouter<A>
where
    A: Actor + 'static,
{
}

impl<A> Handler<AddActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    fn handle(
        &mut self,
        msg: AddActorMessage<A>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        self.route_to.push(msg.actor);
        self.can_route = true;
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<RemoveActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    fn handle(
        &mut self,
        msg: RemoveActorMessage<A>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if let Some(pos) = self
            .route_to
            .iter()
            .position(|x| x.get_address() == msg.actor.get_address())
        {
            self.route_to.remove(pos);
        }
        if self.route_to.len() == 0 {
            self.can_route = false
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<M> for RoundRobinRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(
        &mut self,
        msg: M,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        self.route_index += 1;
        if self.route_index >= self.route_to.len() {
            self.route_index = 0;
        }

        let forward_to = self.route_to.get(self.route_index).unwrap();
        let result = forward_to.send(msg);
        if result.is_err() {
            error!(
                "Could not forward message to target {}",
                forward_to.get_address().actor
            );
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<BulkRouterMessage<M>> for RoundRobinRouter<A>
where
    A: Actor + Handler<BulkActorMessage<M>> + 'static,
    M: BaseActorMessage + 'static,
{
    fn handle(
        &mut self,
        mut msg: BulkRouterMessage<M>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        let total_messages = msg.data.len();
        let total_routees = self.route_to.len();
        let messages_per_routee = total_messages / total_routees;

        for _ in 0..total_routees {
            self.route_index += 1;
            if self.route_index >= self.route_to.len() {
                self.route_index = 0;
            }

            let forward_to = self.route_to.get(self.route_index).unwrap();
            let chunk: Vec<M> = msg.data.drain(0..messages_per_routee).collect();
            let result = forward_to.send(BulkActorMessage::new(chunk));
            if result.is_err() {
                error!(
                    "Could not forward message to target {}",
                    forward_to.get_address().actor
                );
            }
        }
        return Ok(ActorResult::Ok);
    }
}
