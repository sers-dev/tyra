use crate::actor::actor::Actor;
use crate::actor::context::Context;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::system_stop_message::SystemStopMessage;

pub trait Handler<M: ?Sized>
where
    Self: Actor + Sized,
    M: ActorMessage,
{
    fn handle(&mut self, msg: M, context: &Context<Self>);
}

impl<A> Handler<ActorStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: ActorStopMessage, _context: &Context<A>) {
        self.on_actor_stop();
    }
}

impl<A> Handler<SystemStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: SystemStopMessage, _context: &Context<A>) {
        self.on_system_stop();
    }
}
