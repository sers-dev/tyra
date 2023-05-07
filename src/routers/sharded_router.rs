use crate::actor::actor_factory::ActorFactory;

use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::BaseActorMessage;
use crate::prelude::{Actor, ActorMessage, ActorResult, ActorWrapper, BulkActorMessage};
use crate::router::SendToAllTargetsMessage;
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::bulk_router_message::BulkRouterMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use hashring::HashRing;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error::Error;

pub struct ShardedRouter<A>
where
    A: Actor,
{
    num_shards: usize,
    route_to: Vec<ActorWrapper<A>>,
    hash_ring: HashRing<usize>,
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
/// use serde::Serialize;
/// use tyra::router::{ShardedRouterFactory, AddActorMessage};
///
/// // define message
/// #[derive(Hash, Serialize)]
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
            hash_ring: HashRing::new(),
            sharding: HashMap::new(),
            can_route: false,
            stop_on_system_stop,
            stop_on_empty_targets,
        }
    }
}

impl<A> Actor for ShardedRouter<A>
where
    A: Actor,
{
    fn on_system_stop(
        &mut self,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
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
        self.hash_ring.add(self.route_to.len());
        self.route_to.push(msg.actor);
        self.can_route = true;
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
            let _ = self.hash_ring.remove(&pos);
            self.route_to.remove(pos);
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

        let hash = msg.get_hash();
        let target;
        loop {
            let target_id = self.hash_ring.get(&hash);
            if target_id.is_none() {
                warn!("Can't find target for hash.");
                return Ok(ActorResult::Ok);
            }
            let target_id = target_id.unwrap();

            let potential_target = self.route_to.get(target_id.clone());
            if potential_target.is_none() {
                warn!("Target does not exist.");
                return Ok(ActorResult::Ok);
            }
            let potential_target = potential_target.unwrap();
            if !potential_target.is_stopped() {
                target = potential_target;
                break;
            }

            let target_id = target_id.clone();
            self.route_to.remove(target_id.clone());
            self.hash_ring.remove(&target_id);
            if self.route_to.len() == 0 {
                if self.stop_on_empty_targets {
                    debug!("Stopping router, because all targets have been stopped");
                    return Ok(ActorResult::Stop);
                }
                self.can_route = false;
                info!("Router has no valid targets to route to. Dropping message.");
                return Ok(ActorResult::Ok);
            }
        }

        let result = target.send(msg);
        if result.is_err() {
            error!(
                "Could not forward message to target {}",
                target.get_address().actor
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

impl<A, M> Handler<SendToAllTargetsMessage<M>> for ShardedRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: BaseActorMessage + Clone + 'static,
{
    fn handle(
        &mut self,
        msg: SendToAllTargetsMessage<M>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        for i in 0..self.route_to.len() {
            let target = self.route_to.get(i).unwrap();
            if target.is_stopped() {
                self.route_to.remove(i);
            }
        }

        for target in &self.route_to {
            let _ = target.send(msg.msg.clone());
        }

        return Ok(ActorResult::Ok);
    }
}
