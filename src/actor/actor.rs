use std::panic::UnwindSafe;
use crate::prelude::ActorContext;

/// Core trait to define Actors
///
///
/// # Guaranteed Execution Order
///
/// 1. [ActorFactory.new_actor](../prelude/trait.ActorFactory.html#tymethod.new_actor)
/// 2. [pre_start](../prelude/trait.Actor.html#method.pre_start)
/// 3. Start processing [Handler Implementations](../prelude/trait.Handler.html#tymethod.handle)
/// 4. [on_actor_stop](../prelude/trait.Actor.html#method.on_actor_stop)
/// 5. Stops accepting new messages, but will continue to work through all existing Messages in Mailbox
/// 6. [post_stop](../prelude/trait.Actor.html#method.post_stop)
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::{TyraConfig, ActorSystem, Actor, ActorFactory, ActorContext, SerializedMessage, Handler};
///
/// struct TestActor {}
///
/// impl Actor for TestActor {}
///
/// impl Handler<SerializedMessage> for TestActor {
///     fn handle(&mut self, _msg: SerializedMessage, _context: &ActorContext<Self>) {
///     }
/// }
///
/// ```
///
/// # Architecture
///
/// ## Actor Lifecycle
///
/// ```text
///                                 ┌──────────────────────────┐
///                                 │                          │
///                                 │                          │
///                          ┌──────▼──────┐                   │
///                          │             │                   │
///                          │  new_actor  │                   │
///                          │             │                   │
///                          └──────┬──────┘                   │
///                                 │                          │
///                          ┌──────▼──────┐                   │
///                          │             │                   │
///                          │  pre_start  │                   │
///                          │             │                   │
///                          └──────┬──────┘                   │
///                                 │                          │
///                       ┌─────────▼─────────┐                │
///                       │                   ◄─────┐          │
///                       │  handle messages  │     │loop      │panic &&
///                       │                   ├─────┘          │
///                       └──┬──────┬─────┬──▲┘                │RestartPolicy == Always
///                          │      │     │  │                 │
///                          │      │     │  │                 │
///                          │      │     │  │                 │
///         ┌────────────────▼┐     │    ┌▼──┴──────────────┐  │
///         │                 │     │    │                  │  │
///         │  on_actor_stop  │     │    │  on_system_stop  │  │
///         │                 │     │    │                  │  │
///         └────┬────────────┘     │    └──────────────────┘  │
///              │                  │                          │
///              │                  │                          │
///              │                  │                          │
///              │                  │panic                     │
/// ┌────────────▼───────┐          │                          │
/// │                    │   ┌──────▼──────┐                   │
/// │  handle remaining  │   │             │                   │
/// │                    ├───►  post_stop  │                   │
/// │      messages      │   │             │                   │
/// │                    │   └──────┬──────┘                   │
/// └────────────────────┘          │                          │
///                                 │                          │
///                                 │                          │
///                                 └──────────────────────────┘
/// ```
pub trait Actor: Send + Sync + UnwindSafe {
    ///// executed before the first message is handled
    /////
    ///// re-executed after actor restart before first message is handled
    //fn pre_start(&mut self) {}
    ///// executed after the last message is handled
    /////
    ///// also executed in case the actor panics while it handles a message
    //fn post_stop(&mut self) {}
    /// executed when Actor handles internal ActorStopMessage
    ///
    /// After this is called, the Actor will not accept any more messages, but messages within the mailbox will still be processed
    //fn on_actor_stop(&mut self) {}
    fn on_actor_stop(&mut self, _context: &ActorContext<Self>)
        where
            Self: Actor + Sized
    {
        //println!("ON_STOP")

    }
    /// executed when Actor handles internal SystemStopMessage initiated by [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    ///
    /// Default behavior sends an `ActorStopMessage` to all actors which will trigger a clean shutdown
    fn on_system_stop(&mut self, _context: &ActorContext<Self>)
        where
            Self: Actor + Sized
    {}
    ///// // executed when [ActorSystem.send_to_address](../prelude/struct.ActorSystem.html#method.send_to_address) is called
    /////
    ///// # Important Note
    /////
    ///// This is the only function that is not necessarily executed on the thread_pool of the Actor
    ///// It is executed on whatever thread calls [ActorSystem.send_to_address](../prelude/struct.ActorSystem.html#method.send_to_address)
    //fn handle_serialized_message(&self, _msg: SerializedMessage) {}
}
