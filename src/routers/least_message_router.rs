use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::prelude::{Actor, ActorResult};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate::routers::router_message::RouterMessage;
use log::error;
use std::error::Error;

pub struct LeastMessageRouter<A>
where
    A: Actor,
{
    next_route_index: usize,
    min_mailbox_size: usize,
    route_to: Vec<ActorWrapper<A>>,
    can_route: bool,
}

/// implements [ActorFactory](../prelude/trait.ActorFactory.html) to spawn a LeastMessageRouter within an [ActorSystem](../prelude/struct.ActorSystem.html)
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
/// use tyra::router::{LeastMessageRouterFactory, AddActorMessage, RouterMessage};
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
/// let router_factory = LeastMessageRouterFactory::new(15);
/// let router = actor_system
///     .builder()
///     .spawn("router-hello-world", router_factory)
///     .unwrap();
/// router.send(AddActorMessage::new(actor.clone())).unwrap();
/// router.send(RouterMessage::new(FooBar{})).unwrap();
/// ```
pub struct LeastMessageRouterFactory {
    min_mailbox_size: usize
}

impl LeastMessageRouterFactory {
    pub fn new(min_mailbox_size: usize) -> Self {
        Self {
            min_mailbox_size,
        }
    }
}

impl<A> ActorFactory<LeastMessageRouter<A>> for LeastMessageRouterFactory
where
    A: Actor + 'static,
{
    fn new_actor(
        &mut self,
        _context: ActorContext<LeastMessageRouter<A>>,
    ) -> Result<LeastMessageRouter<A>, Box<dyn Error>> {
        return Ok(LeastMessageRouter::new(self.min_mailbox_size));
    }
}

impl<A> LeastMessageRouter<A>
where
    A: Actor,
{
    pub fn new(min_mailbox_size: usize) -> Self {
        Self {
            next_route_index: 0,
            min_mailbox_size,
            route_to: Vec::new(),
            can_route: false,
        }
    }
}

impl<A> Actor for LeastMessageRouter<A> where A: Actor {}

impl<A> Handler<AddActorMessage<A>> for LeastMessageRouter<A>
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
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<RemoveActorMessage<A>> for LeastMessageRouter<A>
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
        }
        if self.route_to.len() == 0 {
            self.can_route = false
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<RouterMessage<M>> for LeastMessageRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(
        &mut self,
        msg: RouterMessage<M>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        let mut target = self.route_to.get(self.next_route_index).unwrap();
        let mut mailbox_size = target.get_mailbox_size();
        let target_len = self.route_to.len();
        for i in 0..target_len {
            if mailbox_size >= self.min_mailbox_size {
                break;
            }
            let potential_target = self.route_to.get(i).unwrap();
            let size = potential_target.get_mailbox_size();
            if size < mailbox_size {
                mailbox_size = size;
                target = potential_target;
            }
        }

        self.next_route_index += 1;
        if self.next_route_index >= target_len {
            self.next_route_index = 0;
        }

        let result = target.send(msg.msg);
        if result.is_err() {
            error!(
                "Could not forward message to target {}",
                target.get_address().actor
            )
        }
        return Ok(ActorResult::Ok);
    }
}
