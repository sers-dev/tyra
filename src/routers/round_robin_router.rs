use std::error::Error;
use log::debug;
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

pub struct RoundRobinRouter<A>
where
    A: Actor,
{
    route_index: usize,
    route_to: Vec<ActorWrapper<A>>,
    can_route: bool,
}

/// implements [ActorFactory](../prelude/trait.ActorFactory.html) to spawn a RoundRobinRouter within an [ActorSystem](../prelude/struct.ActorSystem.html)
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
///     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) -> ActorResult {
///         ActorResult::Ok
///     }
///
/// }
///
/// // create a new actor system with the default config
/// use tyra::router::{RoundRobinRouterFactory, AddActorMessage, RouterMessage};
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
/// let router_factory = RoundRobinRouterFactory::new();
/// let router = actor_system
///     .builder()
///     .spawn("router-hello-world", router_factory)
///     .unwrap();
/// router.send(AddActorMessage::new(actor.clone()));
/// router.send(RouterMessage::new(FooBar{}));
/// ```
pub struct RoundRobinRouterFactory {}

impl RoundRobinRouterFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl<A> ActorFactory<RoundRobinRouter<A>> for RoundRobinRouterFactory
where
    A: Actor + 'static,
{
    fn new_actor(&mut self, _context: ActorContext<RoundRobinRouter<A>>) -> Result<RoundRobinRouter<A>, Box<dyn Error>> {
        return Ok(RoundRobinRouter::new());
    }
}

impl<A> RoundRobinRouter<A>
where
    A: Actor,
{
    pub fn new() -> Self {
        Self {
            route_index: 0,
            route_to: Vec::new(),
            can_route: false,
        }
    }
}

impl<A> Actor for RoundRobinRouter<A> where A: Actor {}

impl<A> Handler<AddActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor,
{
    fn handle(&mut self, msg: AddActorMessage<A>, _context: &ActorContext<Self>) -> ActorResult {
        self.route_to.push(msg.actor);
        self.can_route = true;
        return ActorResult::Ok;
    }
}

impl<A> Handler<RemoveActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor,
{
    fn handle(&mut self, msg: RemoveActorMessage<A>, _context: &ActorContext<Self>) -> ActorResult {
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
        return ActorResult::Ok;
    }
}

impl<A, M> Handler<RouterMessage<M>> for RoundRobinRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(&mut self, msg: RouterMessage<M>, _context: &ActorContext<Self>) -> ActorResult {
        if !self.can_route {
            return ActorResult::Ok;
        }

        self.route_index += 1;
        if self.route_index >= self.route_to.len() {
            self.route_index = 0;
        }

        let forward_to = self.route_to.get(self.route_index).unwrap();
        let result = forward_to.send(msg.msg);
        if result.is_err() {
            debug!("");
        }
        return ActorResult::Ok;
    }
}

impl<A, M> Handler<BulkRouterMessage<M>> for RoundRobinRouter<A>
where
    A: Actor + Handler<BulkActorMessage<M>> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(&mut self, mut msg: BulkRouterMessage<M>, _context: &ActorContext<Self>) -> ActorResult {
        if !self.can_route {
            return ActorResult::Ok;
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
                debug!("ASDF");
            }
        }
        return ActorResult::Ok;
    }
}
