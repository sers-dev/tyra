use crate::actor::actor::ActorTrait;
use crate::message::message::MessageTrait;
use crate::actor::context::Context;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::system_stop_message::SystemStopMessage;

pub trait Handler<M: ?Sized>
    where
        Self: ActorTrait + Sized,
        M: MessageTrait,
{
    fn handle(&mut self, msg: M, context: &Context<Self>);
}

impl<A> Handler<ActorStopMessage> for A
    where
        A: ActorTrait + Sized,
{
    fn handle(&mut self, _msg: ActorStopMessage, _context: &Context<A>) {
        self.on_actor_stop();
    }
}

impl<A> Handler<SystemStopMessage> for A
    where
        A: ActorTrait + Sized,
{
    fn handle(&mut self, _msg: SystemStopMessage, _context: &Context<A>) {
        self.on_system_stop();
    }
}
