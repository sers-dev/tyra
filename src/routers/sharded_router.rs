use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorFactory, ActorMessage, ActorResult, BulkActorMessage};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::bulk_router_message::BulkRouterMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate as tyra;
use log::error;
use std::collections::HashMap;
use std::error::Error;

#[derive(Actor, ActorFactory)]
pub struct ShardedRouter<A>
where
    A: Actor + 'static,
{
    #[tyra(default = "0")]
    num_shards: usize,
    #[tyra(default = "Vec::new()")]
    route_to: Vec<ActorWrapper<A>>,
    #[tyra(default = "HashMap::new()")]
    sharding: HashMap<usize, ActorWrapper<A>>,
    #[tyra(default = "false")]
    can_route: bool,
}

pub use ShardedRouter as ShardedRouterFactory;

impl<A> ShardedRouter<A>
where
    A: Actor + 'static,
{
    fn recalculate_shards(&mut self) {
        let num_routees = self.route_to.len();
        self.num_shards = self.route_to.len() * 5;
        self.sharding.clear();
        for i in 0..self.num_shards {
            let routee = self.route_to.get(i % num_routees).unwrap().clone();
            self.sharding.insert(i, routee);
        }
    }
}

impl<A> Handler<AddActorMessage<A>> for ShardedRouter<A>
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
        self.recalculate_shards();
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<RemoveActorMessage<A>> for ShardedRouter<A>
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
            self.recalculate_shards();
        }
        if self.route_to.len() == 0 {
            self.can_route = false
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<M> for ShardedRouter<A>
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

        let shard_id = msg.get_id() % self.num_shards;
        let forward_to = self.sharding.get(&shard_id).unwrap();
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

impl<A, M> Handler<BulkRouterMessage<M>> for ShardedRouter<A>
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
        let messages_per_routee = total_messages / self.num_shards;

        for i in 0..self.num_shards {
            let forward_to = self.sharding.get(&i).unwrap();
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
