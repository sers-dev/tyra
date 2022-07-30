use crate::actor::base_actor::BaseActor;
use crate::actor::context::ActorContext;
use crate::message::actor_message::ActorMessage;
use crate::message::actor_stop_message::ActorStopMessage;
use crate::message::system_stop_message::SystemStopMessage;
use crate::prelude::{BulkActorMessage, SerializedMessage};

/// Defines which [ActorMessage] is supported per [Actor]
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::{TyraConfig, ActorSystem, BaseActor, ActorFactory, ActorContext, SerializedMessage, ActorMessage, Handler};
///
/// struct TestActor {}
/// impl BaseActor for TestActor {}
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
    Self: BaseActor + Sized,
    M: ActorMessage,
{
    fn handle(&mut self, msg: M, context: &ActorContext<Self>);
}

impl<A> Handler<ActorStopMessage> for A
where
    A: BaseActor + Sized,
{
    fn handle(&mut self, _msg: ActorStopMessage, context: &ActorContext<A>) {
        self.on_actor_stop(context);
    }
}

impl<A> Handler<SystemStopMessage> for A
where
    A: BaseActor + Sized,
{
    fn handle(&mut self, _msg: SystemStopMessage, context: &ActorContext<A>) {
        self.on_system_stop(context);
    }
}

impl<M, A> Handler<BulkActorMessage<M>> for A
where
    Self: BaseActor + Sized,
    A: Handler<M>,
    M: ActorMessage
{
    fn handle(&mut self, msg: BulkActorMessage<M>, context: &ActorContext<Self>) {
        for i in msg.data.into_iter() {
            self.handle(i, context);
        }
    }
}

impl<A> Handler<SerializedMessage> for A
    where
        A: BaseActor + Sized + Actor,
{
    fn handle(&mut self, msg: SerializedMessage, context: &ActorContext<A>) {
        self.handle_serialized_message(msg, context);
    }
}


impl<A> BaseActor for A
    where
        A: Actor {
    fn on_actor_stop(&mut self, context: &ActorContext<Self>) where Self: BaseActor + Sized {
        self.on_actor_stop_int(context);
    }

    fn on_system_stop(&mut self, context: &ActorContext<Self>) where Self: BaseActor + Sized {
        self.on_system_stop_int(context);
    }
}

pub trait Actor: BaseActor
{

    fn handle_serialized_message(&mut self, _msg: SerializedMessage, _context: &ActorContext<Self>)
        where Self: BaseActor + Sized + 'static {
        println!("ASDF")
    }


    fn handle_serialized_message_old<A>(&mut self, _msg: SerializedMessage, _context: &ActorContext<A>)
    where A: BaseActor + Sized {
        println!("ASDF")
    }

    /// executed before the first message is handled
    ///
    /// re-executed after actor restart before first message is handled
    fn pre_start(&mut self, _context: &ActorContext<Self>)
    where
        Self: BaseActor + Sized
    {
        println!("PRE_START")
    }

    fn post_stop(&mut self, _context: &ActorContext<Self>)
    where
        Self: BaseActor + Sized
    {
        println!("POST_STOP")

    }

    fn on_actor_stop_int(&mut self, _context: &ActorContext<Self>)
        where
            Self: BaseActor + Sized
    {
        println!("ON_STOP ActorMessageDeserializer")

    }

    fn on_system_stop_int(&mut self, context: &ActorContext<Self>)
        where
            Self: BaseActor + Sized
    {
        println!("ON_SYS_STOP ActorMessageDeserializer");
        context.actor_ref.send(ActorStopMessage{});

    }

}
//impl<A> ActorMessageDeserializer for A
//where
//    A: Actor + Sized,
//{
//    fn handle_serialized_message<B>(&mut self, _msg: SerializedMessage, _context: &ActorContext<B>)
//        where B: Actor + Sized {
//        println!("NA SERS");
//    }
//
//}