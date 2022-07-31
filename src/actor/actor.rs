use crate::prelude::{ActorContext, ActorStopMessage, SerializedMessage};
use std::panic::UnwindSafe;

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
/// use tyra::prelude::{TyraConfig, ActorSystem, ActorFactory, ActorContext, SerializedMessage, Handler, Actor};
///
/// struct TestActor {}
///
/// impl Actor for TestActor {}
///
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
pub trait Actor: Send + Sync + UnwindSafe + Sized {
    /// executed whenever Actor receives a [SerializedMessage](../prelude/struct.SerializedMessage.html)
    fn handle_serialized_message(
        &mut self,
        _msg: SerializedMessage,
        _context: &ActorContext<Self>,
    ) {
    }

    /// executed before the first message is handled
    ///
    /// re-executed after actor restart before first message is handled
    fn pre_start(&mut self, _context: &ActorContext<Self>) {}

    /// executed after the last message is handled
    ///
    /// also executed in case the actor panics while it handles a message
    fn post_stop(&mut self, _context: &ActorContext<Self>) {}

    /// executed when Actor handles internal ActorStopMessage
    ///
    /// After this is called, the Actor will not accept any more messages, but messages within the mailbox will still be processed
    fn on_actor_stop(&mut self, _context: &ActorContext<Self>) {}

    /// executed when Actor handles internal SystemStopMessage initiated by [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    ///
    /// Default behavior sends an `ActorStopMessage` to all actors which will trigger a clean shutdown
    fn on_system_stop(&mut self, context: &ActorContext<Self>) {
        context.actor_ref.send(ActorStopMessage {});
    }
}
