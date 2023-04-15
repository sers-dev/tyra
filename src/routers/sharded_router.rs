use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorMessage, ActorResult, BulkActorMessage};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::bulk_router_message::BulkRouterMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use log::{debug, error};
use std::collections::HashMap;
use std::error::Error;

pub struct ShardedRouter<A>
where
    A: Actor,
{
    num_shards: usize,
    route_to: Vec<ActorWrapper<A>>,
    sharding: HashMap<usize, ActorWrapper<A>>,
    can_route: bool,
    stop_on_system_stop: bool,
    stop_on_empty_targets: bool,
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
/// use tyra::router::{ShardedRouterFactory, AddActorMessage};
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
/// let router_factory = ShardedRouterFactory::new(true, true);
/// let router = actor_system
///     .builder()
///     .spawn("router-hello-world", router_factory)
///     .unwrap();
/// router.send(AddActorMessage::new(actor.clone())).unwrap();
/// router.send(FooBar{}).unwrap();
/// ```
pub struct ShardedRouterFactory {
    /// defines if the actor should automatically be stopped when the system is stopped. If set to false it's up to the user to setup their own shutdown process if they want a quick and clean exit
    stop_on_system_stop: bool,
    /// defines if the actor should automatically be stopped if it receives a message after all targets have been automatically removed
    /// this does not apply if the last target has been removed through a `RemoveActorMessage
    stop_on_empty_targets: bool,
}

impl ShardedRouterFactory {
    pub fn new(stop_on_system_stop: bool, stop_on_empty_targets: bool) -> Self {
        Self {
            stop_on_system_stop,
            stop_on_empty_targets,
        }
    }
}

impl<A> ActorFactory<ShardedRouter<A>> for ShardedRouterFactory
where
    A: Actor + 'static,
{
    fn new_actor(
        &mut self,
        _context: ActorContext<ShardedRouter<A>>,
    ) -> Result<ShardedRouter<A>, Box<dyn Error>> {
        return Ok(ShardedRouter::new(
            self.stop_on_system_stop,
            self.stop_on_empty_targets,
        ));
    }
}

impl<A> ShardedRouter<A>
where
    A: Actor,
{
    pub fn new(stop_on_system_stop: bool, stop_on_empty_targets: bool) -> Self {
        Self {
            num_shards: 0,
            route_to: Vec::new(),
            sharding: HashMap::new(),
            can_route: false,
            stop_on_system_stop,
            stop_on_empty_targets,
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

impl<A> Actor for ShardedRouter<A> where A: Actor {
    fn on_system_stop(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        if self.stop_on_system_stop {
            let result = context.actor_ref.stop();
            if result.is_err() {
                error!(
                    "Could not forward message ActorStopMessage to target {}",
                    context.actor_ref.get_address().actor
                );
                return Ok(ActorResult::Stop);
            }
        }
        return Ok(ActorResult::Ok);

    }
}

impl<A> Handler<AddActorMessage<A>> for ShardedRouter<A>
where
    A: Actor,
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
    A: Actor,
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

        let mut shard_id = msg.get_id() % self.num_shards;
        let mut forward_to = self.sharding.get(&shard_id).unwrap();
        loop {
            if !forward_to.is_stopped() {
                break;
            }
            self.route_to.remove(shard_id);
           if self.route_to.len() == 0 && self.stop_on_empty_targets {
               debug!("Stopping router, because all targets have been removed");
               return Ok(ActorResult::Stop)
           }
            self.recalculate_shards();
            shard_id = msg.get_id() % self.num_shards;
            forward_to = self.sharding.get(&shard_id).unwrap();
        }

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

        for i in 0..self.route_to.len() {
            let target = self.route_to.get(i).unwrap();
            if target.is_stopped() {
                self.route_to.remove(i);
                self.recalculate_shards();
            }
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
