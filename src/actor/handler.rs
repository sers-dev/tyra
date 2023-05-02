use crate::actor::actor::Actor;
use crate::actor::context::ActorContext;
use crate::message::actor_message::BaseActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::sleep_message::SleepMessage;
use crate::message::system_stop_message::SystemStopMessage;
use crate::prelude::{ActorResult, BulkActorMessage, SerializedMessage};
use std::error::Error;

/// Defines which [ActorMessage] is supported per [Actor]
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use std::error::Error;
/// use serde::Serialize;
/// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor, ActorResult, ActorMessage};
///
/// struct TestActor {}
/// impl Actor for TestActor {}
///
/// #[derive(Hash, Serialize)]
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
///
/// impl Handler<FooBar> for TestActor {
///     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
///         Ok(ActorResult::Ok)
///     }
/// }
/// ```
pub trait Handler<M: ?Sized>
where
    Self: Actor + Sized,
    M: BaseActorMessage,
{
    fn handle(
        &mut self,
        msg: M,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>>;
}

impl<A> Handler<ActorStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(
        &mut self,
        _msg: ActorStopMessage,
        _context: &ActorContext<A>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        return Ok(ActorResult::Stop);
    }
}

impl<A> Handler<SystemStopMessage> for A
where
    A: Actor + Sized,
{
    fn handle(
        &mut self,
        _msg: SystemStopMessage,
        context: &ActorContext<A>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        return self.on_system_stop(context);
    }
}

impl<M, A> Handler<BulkActorMessage<M>> for A
where
    Self: Actor + Sized,
    A: Handler<M>,
    M: BaseActorMessage,
{
    fn handle(
        &mut self,
        msg: BulkActorMessage<M>,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        for i in msg.data.into_iter() {
            self.handle(i, context)?;
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<SerializedMessage> for A
where
    A: Actor + Sized + Actor,
{
    fn handle(
        &mut self,
        msg: SerializedMessage,
        context: &ActorContext<A>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        return self.handle_serialized_message(msg, context);
    }
}

impl<A> Handler<SleepMessage> for A
where
    A: Actor + Sized + Actor,
{
    fn handle(
        &mut self,
        msg: SleepMessage,
        _context: &ActorContext<A>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        return Ok(ActorResult::Sleep(msg.duration));
    }
}
