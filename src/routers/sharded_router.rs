use std::collections::HashMap;
use std::error::Error;
use log::error;
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::prelude::{Actor, ActorResult, BulkActorMessage};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::bulk_router_message::BulkRouterMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate::routers::router_message::RouterMessage;

pub struct ShardedRouter<A>
    where
        A: Actor,
{
    num_shards: usize,
    route_to: Vec<ActorWrapper<A>>,
    sharding: HashMap<usize, ActorWrapper<A>>,
    can_route: bool,
}

/// implements [ActorFactory](../prelude/trait.ActorFactory.html) to spawn a ShardedRouter within an [ActorSystem](../prelude/struct.ActorSystem.html)
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use std::error::Error;
/// use tyra::prelude::*;
/// use std::process::exit;
/// use std::time::Duration;
///
/// // define message
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
///
/// // define actor
/// struct HelloWorld {}
/// impl Actor for HelloWorld {}
///
/// // setup required Factory
/// struct HelloWorldFactory {}
/// impl ActorFactory<HelloWorld> for HelloWorldFactory {
///     fn new_actor(&mut self, _context: ActorContext<HelloWorld>) -> Result<HelloWorld, Box<dyn Error>> {
///         Ok(HelloWorld {})
///     }
/// }
///
/// // setup Message Handler for Actor
/// impl Handler<FooBar> for HelloWorld {
///     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
///         Ok(ActorResult::Ok)
///     }
///
/// }
///
/// // create a new actor system with the default config
/// use tyra::router::{ShardedRouterFactory, AddActorMessage, RouterMessage};
/// let actor_config = TyraConfig::new().unwrap();
/// let actor_system = ActorSystem::new(actor_config);
///
/// // create the actor
/// let actor_factory = HelloWorldFactory {};
/// let actor = actor_system
///     .builder()
///     .spawn("hello-world", actor_factory)
///     .unwrap();
///
/// // create the router, fill it, and route a message
/// let router_factory = ShardedRouterFactory::new();
/// let router = actor_system
///     .builder()
///     .spawn("router-hello-world", router_factory)
///     .unwrap();
/// router.send(AddActorMessage::new(actor.clone()));
/// router.send(RouterMessage::new(FooBar{}));
/// ```
pub struct ShardedRouterFactory {}

impl ShardedRouterFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl<A> ActorFactory<ShardedRouter<A>> for ShardedRouterFactory
    where
        A: Actor + 'static,
{
    fn new_actor(&mut self, _context: ActorContext<ShardedRouter<A>>) -> Result<ShardedRouter<A>, Box<dyn Error>> {
        return Ok(ShardedRouter::new());
    }
}

impl<A> ShardedRouter<A>
    where
        A: Actor,
{
    pub fn new() -> Self {
        Self {
            num_shards: 0,
            route_to: Vec::new(),
            sharding: HashMap::new(),
            can_route: false,
        }
    }

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

impl<A> Actor for ShardedRouter<A> where A: Actor {}

impl<A> Handler<AddActorMessage<A>> for ShardedRouter<A>
    where
        A: Actor,
{
    fn handle(&mut self, msg: AddActorMessage<A>, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        self.route_to.push(msg.actor);
        self.can_route = true;
        self.recalculate_shards();
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<RemoveActorMessage<A>> for ShardedRouter<A>
    where
        A: Actor,
{
    fn handle(&mut self, msg: RemoveActorMessage<A>, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
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

impl<A, M> Handler<RouterMessage<M>> for ShardedRouter<A>
    where
        A: Actor + Handler<M> + 'static,
        M: ActorMessage + 'static,
{
    fn handle(&mut self, msg: RouterMessage<M>, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        let shard_id = msg.get_id() % self.num_shards;
        let forward_to = self.sharding.get(&shard_id).unwrap();
        let result = forward_to.send(msg.msg);
        if result.is_err() {
            error!("Could not forward message {} to target {}", msg.get_id(), forward_to.get_address().actor);
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<BulkRouterMessage<M>> for ShardedRouter<A>
    where
        A: Actor + Handler<BulkActorMessage<M>> + 'static,
        M: ActorMessage + 'static,
{
    fn handle(&mut self, mut msg: BulkRouterMessage<M>, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
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
                error!("Could not forward message {} to target {}", msg.get_id(), forward_to.get_address().actor);
            }
        }
        return Ok(ActorResult::Ok);
    }
}
