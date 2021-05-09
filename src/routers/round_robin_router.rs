use crate::actor::actor::Actor;
use crate::actor::actor_factory::ActorFactory;
use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::message::actor_message::ActorMessage;
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate::routers::router_message::RouterMessage;

pub struct RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    context: ActorContext<Self>,
    route_index: usize,
    route_to: Vec<ActorWrapper<A>>,
    can_route: bool,
}

/// implements [ActorFactory](../prelude/trait.ActorFactory.html) to spawn a RoundRobinRouter within an [ActorSystem](../prelude/struct.ActorSystem.html)
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
    fn new_actor(&self, context: ActorContext<RoundRobinRouter<A>>) -> RoundRobinRouter<A> {
        RoundRobinRouter::new(context)
    }
}

impl<A> RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    pub fn new(context: ActorContext<Self>) -> Self {
        Self {
            context,
            route_index: 0,
            route_to: Vec::new(),
            can_route: false,
        }
    }
}

impl<A> Actor for RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    fn on_system_stop(&mut self) {
        self.context.actor_ref.stop();
    }
}

impl<A> Handler<AddActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    fn handle(&mut self, msg: AddActorMessage<A>, _context: &ActorContext<Self>) {
        self.route_to.push(msg.actor);
        self.can_route = true;
    }
}

impl<A> Handler<RemoveActorMessage<A>> for RoundRobinRouter<A>
where
    A: Actor + 'static,
{
    fn handle(&mut self, msg: RemoveActorMessage<A>, _context: &ActorContext<Self>) {
        if let Some(pos) = self
            .route_to
            .iter()
            .position(|x| x.get_address() == msg.actor.get_address())
        {
            self.route_to.remove(pos);
        }
    }
}

impl<A, M> Handler<RouterMessage<M>> for RoundRobinRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(&mut self, msg: RouterMessage<M>, _context: &ActorContext<Self>) {
        if !self.can_route {
            return;
        }

        self.route_index += 1;
        if self.route_index >= self.route_to.len() {
            self.route_index = 0;
        }

        let forward_to = self.route_to.get(self.route_index).unwrap();
        forward_to.send(msg.msg);
    }
}
