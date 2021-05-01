use crate::actor::actor_ref::ActorRef;
use crate::actor::context::Context;
use crate::actor::actor::ActorTrait;
use crate::message::message::MessageTrait;
use crate::actor::handler::Handler;
use crate::actor::props::ActorProps;

pub struct RouterMessage<M>
where
    M: MessageTrait + 'static,
{
    pub msg: M
}
impl<M> MessageTrait for RouterMessage<M>
    where
        M: MessageTrait + 'static,
{}

impl<M> RouterMessage<M>
    where
        M: MessageTrait + 'static,
{
    pub fn new(msg: M) -> Self {
        Self {
            msg
        }
    }
}

pub struct AddActorMessage<A>
where
    A: ActorTrait + 'static,
{
    actor: ActorRef<A>
}

impl<A> AddActorMessage<A>
    where
        A: ActorTrait + 'static,
{
    pub fn new(actor: ActorRef<A>) -> Self {
        Self {
            actor,
        }
    }
}

impl<A> MessageTrait for AddActorMessage<A>
    where
        A: ActorTrait + 'static,
{}

pub struct RemoveActorMessage<A>
    where
        A: ActorTrait + 'static,
{
    actor: ActorRef<A>
}

impl<A> RemoveActorMessage<A>
    where
        A: ActorTrait + 'static,
{
    pub fn new(actor: ActorRef<A>) -> Self {
        Self {
            actor,
        }
    }
}
impl<A> MessageTrait for RemoveActorMessage<A>
    where
        A: ActorTrait + 'static,
{}


pub struct RoundRobinRouter<A>
    where
        A: ActorTrait + 'static,
{
    context: Context<Self>,
    route_index: usize,
    route_to: Vec<ActorRef<A>>,
    can_route: bool,
}

pub struct RoundRobinRouterProps {}

impl RoundRobinRouterProps {
    pub fn new() -> Self {
        Self{}
    }
}

impl<A> ActorProps<RoundRobinRouter<A>> for RoundRobinRouterProps
    where
        A: ActorTrait + 'static,
{
    fn new_actor(&self, context: Context<RoundRobinRouter<A>>) -> RoundRobinRouter<A> {
        RoundRobinRouter::new(context)
    }

}

impl<A> RoundRobinRouter<A>
    where
        A: ActorTrait + 'static,
{
    pub fn new(context: Context<Self>) -> Self {
        Self {
            context,
            route_index: 0,
            route_to: Vec::new(),
            can_route: false,
        }
    }
}

impl<A> ActorTrait for RoundRobinRouter<A>
    where
        A: ActorTrait + 'static,
{
    fn on_system_stop(&mut self) {
        self.context.actor_ref.stop();
    }
}

impl<A> Handler<AddActorMessage<A>> for RoundRobinRouter<A>
    where
        A: ActorTrait + 'static,
{
    fn handle(&mut self, msg: AddActorMessage<A>, _context: &Context<Self>) {
        self.route_to.push(msg.actor);
        self.can_route = true;
    }
}

impl<A> Handler<RemoveActorMessage<A>> for RoundRobinRouter<A>
    where
        A: ActorTrait + 'static,
{
    fn handle(&mut self, msg: RemoveActorMessage<A>, _context: &Context<Self>) {
        if let Some(pos) = self.route_to.iter().position(|x| x.get_address() == msg.actor.get_address()) {
            self.route_to.remove(pos);
        }
    }
}

impl<A, M> Handler<RouterMessage<M>> for RoundRobinRouter<A>
    where
        A: ActorTrait + Handler<M> + 'static,
        M: MessageTrait + 'static
{
    fn handle(&mut self, msg: RouterMessage<M>, _context: &Context<Self>) {
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