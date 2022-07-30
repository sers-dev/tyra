use crate::actor::base_actor::BaseActor;
use crate::prelude::{ActorContext, ActorStopMessage, SerializedMessage};

impl<A> BaseActor for A
    where
        A: Actor {
    fn actor_stop(&mut self, context: &ActorContext<Self>) where Self: BaseActor + Sized {
        self.on_actor_stop(context);
    }

    fn system_stop(&mut self, context: &ActorContext<Self>) where Self: BaseActor + Sized {
        self.on_system_stop(context);
    }
}

pub trait Actor: BaseActor + Sized
{

    fn handle_serialized_message(&mut self, _msg: SerializedMessage, _context: &ActorContext<Self>)
    {
        println!("ASDF")
    }


    /// executed before the first message is handled
    ///
    /// re-executed after actor restart before first message is handled
    fn pre_start(&mut self, _context: &ActorContext<Self>)
    {
        println!("PRE_START")
    }

    fn post_stop(&mut self, _context: &ActorContext<Self>)
    {
        println!("POST_STOP")

    }

    fn on_actor_stop(&mut self, _context: &ActorContext<Self>)
    {
        println!("ON_STOP ActorMessageDeserializer")

    }

    fn on_system_stop(&mut self, context: &ActorContext<Self>)
    {
        println!("ON_SYS_STOP ActorMessageDeserializer");
        context.actor_ref.send(ActorStopMessage{});

    }

}
