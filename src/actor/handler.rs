use crate::actor::actor::Actor;
use crate::actor::context::ActorContext;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::system_stop_message::SystemStopMessage;
use crate::prelude::{ActorResult, BulkActorMessage, SerializedMessage};

/// Defines which [ActorMessage] is supported per [Actor]
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, ActorMessage, Handler, Actor, ActorResult};
///
/// struct TestActor {}
/// impl Actor for TestActor {}
///
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
///
/// impl Handler<FooBar> for TestActor {
///     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) -> ActorResult {
///         ActorResult::Ok
///     }
/// }
/// ```
pub trait Handler<M: ?Sized>
where
    Self: Actor + Sized,
    M: ActorMessage,
{
    fn handle(&mut self, msg: M, context: &ActorContext<Self>) -> ActorResult;
}

impl<A> Handler<ActorStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: ActorStopMessage, context: &ActorContext<A>) -> ActorResult {
        let actor_result = self.on_actor_stop(context);
        if actor_result != ActorResult::Stop || actor_result != ActorResult::Kill {
            return ActorResult::Stop;
        }
        return actor_result;
    }
}

impl<A> Handler<SystemStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(&mut self, _msg: SystemStopMessage, context: &ActorContext<A>) -> ActorResult {
        return self.on_system_stop(context);
    }
}

impl<M, A> Handler<BulkActorMessage<M>> for A
where
    Self: Actor + Sized,
    A: Handler<M>,
    M: ActorMessage,
{
    fn handle(&mut self, msg: BulkActorMessage<M>, context: &ActorContext<Self>) -> ActorResult {
        for i in msg.data.into_iter() {
            self.handle(i, context);
        }
        return ActorResult::Ok;
    }
}

impl<A> Handler<SerializedMessage> for A
where
    A: Actor + Sized + Actor,
{
    fn handle(&mut self, msg: SerializedMessage, context: &ActorContext<A>) -> ActorResult {
        return self.handle_serialized_message(msg, context);
    }
}
