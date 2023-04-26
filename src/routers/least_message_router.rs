use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::prelude::{Actor, ActorMessage, ActorResult};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use log::{debug, error};
use std::error::Error;
use crate::message::actor_message::BaseActorMessage;
use crate::router::SendToAllTargetsMessage;

pub struct LeastMessageRouter<A>
where
    A: Actor,
{
    next_route_index: usize,
    min_mailbox_size: usize,
    route_to: Vec<ActorWrapper<A>>,
    can_route: bool,
    stop_on_system_stop: bool,
    stop_on_empty_targets: bool,
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
/// use tyra::router::{LeastMessageRouterFactory, AddActorMessage};
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
/// let router_factory = LeastMessageRouterFactory::new(15, true, true);
/// let router = actor_system
///     .builder()
///     .spawn("router-hello-world", router_factory)
///     .unwrap();
/// router.send(AddActorMessage::new(actor.clone())).unwrap();
/// router.send(FooBar{}).unwrap();
/// ```
pub struct LeastMessageRouterFactory {
    /// minimum mailbox size, that needs to be exceeded to actually search for the router with the least messages
    min_mailbox_size: usize,
    /// defines if the actor should automatically be stopped when the system is stopped. If set to false it's up to the user to setup their own shutdown process if they want a quick and clean exit
    stop_on_system_stop: bool,
    /// defines if the actor should automatically be stopped if it receives a message after all targets have been automatically removed
    /// this does not apply if the last target has been removed through a `RemoveActorMessage`
    stop_on_empty_targets: bool,
}

impl LeastMessageRouterFactory {
    pub fn new(min_mailbox_size: usize, stop_on_system_stop: bool, stop_on_empty_targets: bool,) -> Self {
        Self {
            min_mailbox_size,
            stop_on_system_stop,
            stop_on_empty_targets,
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
        return Ok(LeastMessageRouter::new(self.min_mailbox_size, self.stop_on_system_stop, self.stop_on_empty_targets));
    }
}

impl<A> LeastMessageRouter<A>
where
    A: Actor,
{
    pub fn new(min_mailbox_size: usize, stop_on_system_stop: bool, stop_on_empty_targets: bool,) -> Self {
        Self {
            next_route_index: 0,
            min_mailbox_size,
            route_to: Vec::new(),
            can_route: false,
            stop_on_system_stop,
            stop_on_empty_targets,
        }
    }
}

impl<A> Actor for LeastMessageRouter<A> where A: Actor {
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

impl<A, M> Handler<M> for LeastMessageRouter<A>
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

        let mut target;
        // skip/remove stopped actors
        loop {
            let route_index = self.next_route_index;
            target = self.route_to.get(self.next_route_index).unwrap();

            if target.is_stopped() {
                self.next_route_index += 1;
                if self.next_route_index >= (self.route_to.len() - 1) {
                    self.next_route_index = 0;
                }
                self.route_to.remove(route_index);
                if self.route_to.len() == 0 && self.stop_on_empty_targets {
                    debug!("Stopping router, because all targets have been removed");
                    return Ok(ActorResult::Stop)
                }
            }
            else {
                break;
            }
        }

        let mut mailbox_size = target.get_mailbox_size();
        let target_len = self.route_to.len();
        for i in 0..target_len {
            if mailbox_size <= self.min_mailbox_size {
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

        let result = target.send(msg);
        if result.is_err() {
            error!(
                "Could not forward message to target {}",
                target.get_address().actor
            )
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<SendToAllTargetsMessage<M>> for LeastMessageRouter<A>
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

        // skip/remove stopped actors
        loop {
            let route_index = self.next_route_index;
            let target = self.route_to.get(self.next_route_index).unwrap();

            if target.is_stopped() {
                self.next_route_index += 1;
                if self.next_route_index >= (self.route_to.len() - 1) {
                    self.next_route_index = 0;
                }
                self.route_to.remove(route_index);
                if self.route_to.len() == 0 && self.stop_on_empty_targets {
                    debug!("Stopping router, because all targets have been removed");
                    return Ok(ActorResult::Stop)
                }
            }
            else {
                break;
            }
        }

        for target in &self.route_to {
            let _ =  target.send(msg.msg.clone());
        }

        return Ok(ActorResult::Ok);
    }
}