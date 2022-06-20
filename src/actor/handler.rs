use crate::actor::actor::Actor;
use crate::actor::context::ActorContext;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::system_stop_message::SystemStopMessage;
use crate::prelude::BulkActorMessage;

/// Defines which [ActorMessage] is supported per [Actor]
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::{TyraConfig, ActorSystem, Actor, ActorFactory, ActorContext, SerializedMessage, ActorMessage, Handler};
///
/// struct TestActor {}
/// impl Actor for TestActor {}
///
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
///
/// impl Handler<FooBar> for TestActor {
///     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) {
///     }
/// }
/// ```
pub trait Handler<M: ?Sized>
where
    Self: Actor + Sized,
    M: ActorMessage,
{
    fn handle(&mut self, msg: M, context: &ActorContext<Self>);
}

impl<A> Handler<ActorStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: ActorStopMessage, _context: &ActorContext<A>) {
        self.on_actor_stop();
    }
}

impl<A> Handler<SystemStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: SystemStopMessage, _context: &ActorContext<A>) {
        self.on_system_stop();
    }
}

impl<M, A> Handler<BulkActorMessage<M>> for A
where
    Self: Actor + Sized,
    A: Handler<M>,
    M: ActorMessage
{
    fn handle(&mut self, msg: BulkActorMessage<M>, context: &ActorContext<Self>) {
        for i in msg.data.into_iter() {
            self.handle(i, context);
        }
    }
}