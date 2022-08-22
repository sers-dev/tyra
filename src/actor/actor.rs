use crate::prelude::{ActorContext, ActorPanicSource, ActorResult, SerializedMessage};
use std::panic::UnwindSafe;
use crate::message::actor_stop_message::ActorStopMessage;

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
///        ┌──────────────────────────────────────────────────────────────────────────────────┐
///        │                                                                                  │
///        │                                        ┌───┐                                     │
///        │                                        │   │                                     │
///        │                              ┌─────────▼───┴──────────┐                          │
///        │       ┌──────────────────────┤                        │                          │
///        │       │                      │  Handler<M>::handle()  ◄───────────┐              │
///        │       │       ┌──────────────►                        │           │              │
///        │       │       │              └───────────┬────────────┤           │              │
///        │       │       │                          │            │           │              │
///        │       │       │                          │            │           │              │
///        │       │       │                          │            │           │              │
///        │       │       │                          │            │           │              │
///     ┌──▼───────▼───────┴──┐           ┌───────────▼──────────┐ │        ┌──┴──────────────┴──┐
/// ┌───►                     │           │                      │          │                    │
/// │   │  Actor::on_panic()  ├───────────►  Actor::post_stop()  ◄──────────┤  Actor::pre_start  │
/// └───┤                     │           │                      │          │                    │
///     └────────┬─▲──────────┘           └───────────▲──────────┘ │        └──▲──────────────┬──┘
///              │ │                                  │            │           │              │
///              │ │                                  │            │           │              │
///              │ │                                  │            │           │              │
///              │ │                                  │            │           │              │
///              │ │                   ┌──────────────┴────────────▼─┐         │              │
///              │ └───────────────────┤                             ├─────────┘              │
///              │                     │  ActorFactory::new_actor()  │                        │
///              └─────────────────────►                             ◄────────────────────────┘
///                                    └─────────────────────────────┘
///
///  ┌───┬────────────────────────────────┬────────────────────────┬───────────────────────────┬───────────────────────────┬───────────────────────────────┐
///  │   │                                │                        │                           │                           │                               │
///  │ ┌─▼──────────────────────┐       ┌─▼──────────┐           ┌─▼─────────────────┐       ┌─▼──────────────────┐      ┌─▼─────────────────────────┐  ┌──▼──────────────────┐
///  │ │                        │       │            │           │                   │       │                    │      │                           │  │                     │
///  │ │  ActorResult::Restart  │       │  panic!()  │           │  ActorResult::Ok  │       │ ActorResult::Stop  │      │  ActorResult::Sleep(dur)  │  │  ActorResult::Kill  │
///  │ │                        │       │            │           │                   │       │                    │      │                           │  │                     │
///  │ └─┬──────────────────────┘       └─┬──────────┘           └─┬─────────────────┘       └─┬──────────────────┘      └─┬─────────────────────────┘  └──┬──────────────────┘
///  │   │                                │                        │                           │                           │                               │
///  │   │                                │                        │                           │                           │                               │
///  │ ┌─▼───────────────────────────┐  ┌─▼───────────────────┐  ┌─▼──────────────────────┐  ┌─▼──────────────────────┐  ┌─▼──────────────────────┐     ┌──▼───────────────────┐
///  │ │                             │  │                     │  │                        │  │                        │  │                        │     │                      │
///  │ │  ActorFactory::new_actor()  │  │  Actor::on_panic()  │  │  Handler<M>::handle()  │  │  Handler<M>::handle()  │  │  Handler<M>::handle()  │     │  Actor::post_stop()  │
///  │ │                             │  │                     │  │                        │  │                        │  │                        │     │                      │
///  │ │  Actor::pre_start()         │  └─┬───────────────────┘  └─┬──────────────────────┘  └─┬──────────────────────┘  └─┬──────────────────────┘     └──▲───────────────────┘
///  │ │                             │    │                        │                           │                           │                               │
///  │ └─┬───────────────────────────┘    │                        │                           │                           │                               │
///  │   │                                │                        │                         ┌─▼─────────────────┐         │                               │
///  │   │                                │                        └─────────────────────────►                   ◄─────────┘                               │
///  │   │                                │                                                  │  is_stopped &&    │                                         │
///  │   │                                │                                                  │                   │                                         │
///  │   │                                │                                                  │  mailbox.empty()  │                                         │
///  │   │                                │                                                  │                   │                                         │
///  │   │                                │                                                  │  false      true  │                                         │
///  │   │                                │                                                  │                   │                                         │
///  │   │                                │                                                  └──┬────────────┬───┘                                         │
///  │   │                                │                                                     │            │                                             │
///  └─◄─┴──────────────────────────────◄─┴─────────────────────────────────────────────────────┘            └─────────────────────────────────────────────┘
///
/// ```
pub trait Actor: Send + Sync + UnwindSafe + Sized {
    /// executed whenever Actor receives a [SerializedMessage](../prelude/struct.SerializedMessage.html)
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    fn handle_serialized_message(
        &mut self,
        _msg: SerializedMessage,
        _context: &ActorContext<Self>,
    ) -> ActorResult {
        return ActorResult::Ok;
    }

    /// executed whenever a panic occurs within the actor
    ///
    /// determines actor behavior in case of a panic based on return value
    /// WARNING: returning `ActorResult::Restart` if `source == ActorPanicSource::Restart` can potentially result in an endless loop that will block the thread the actor is running on, until the restart was successful
    /// NOTE: if this function panics it will trigger a second time with `source == ActorPanicSource::OnPanic`, if another panic occurs in that case, the actor will be stopped immediately without finishing the mailbox!
    fn on_panic(&mut self, _context: &ActorContext<Self>, source: ActorPanicSource) -> ActorResult {
        return match source {

            ActorPanicSource::PreStart => {
                ActorResult::Ok
            }
            ActorPanicSource::Message => {
                ActorResult::Restart
            }
            ActorPanicSource::Restart => {
                ActorResult::Ok
            }
            ActorPanicSource::OnPanic => {
                ActorResult::Kill
            }
        }
    }

    /// executed before the first message is handled
    ///
    /// re-executed after actor restart before first message is handled
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::PreStart`
    fn pre_start(&mut self, _context: &ActorContext<Self>) -> ActorResult {
        return ActorResult::Ok;
    }

    /// executed after the last message is handled
    ///
    /// also executed in case the actor panics while it handles a message
    fn post_stop(&mut self, _context: &ActorContext<Self>) {}

    /// executed when Actor handles internal ActorStopMessage
    ///
    /// If the return value is neither ActorResult::Stop nor ActorResult::StopImmediately the return value will internally be converted to ActorResult::Stop
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    fn on_actor_stop(&mut self, _context: &ActorContext<Self>) -> ActorResult {
        return ActorResult::Stop;
    }

    /// executed when Actor handles internal SystemStopMessage initiated by [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    ///
    /// Default behavior sends an `ActorStopMessage` to all actors which will trigger a clean shutdown
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    fn on_system_stop(&mut self, context: &ActorContext<Self>) -> ActorResult {
        context.actor_ref.send(ActorStopMessage {});
        return ActorResult::Ok;
    }
}
