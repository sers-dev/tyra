use std::error::Error;
use crate::prelude::{ActorContext, ActorPanicSource, ActorResult, SerializedMessage};
use std::panic::UnwindSafe;
use log::{debug, error};
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
///   ┌───┬────────────────────────────────┬────────────────────────┬────────────────────────┬───────────────────────────┬───────────────────────────┬───────────────────────────────┐
///   │   │                                │                        │                        │                           │                           │                               │
///   │ ┌─▼──────────────────────┐       ┌─▼──────────┐           ┌─▼────────────────┐     ┌─▼─────────────────┐       ┌─▼──────────────────┐      ┌─▼─────────────────────────┐  ┌──▼──────────────────┐
///   │ │                        │       │            │           │                  │     │                   │       │                    │      │                           │  │                     │
///   │ │  ActorResult::Restart  │       │  panic!()  │           │  Box<dyn Error>  │     │  ActorResult::Ok  │       │ ActorResult::Stop  │      │  ActorResult::Sleep(dur)  │  │  ActorResult::Kill  │
///   │ │                        │       │            │           │                  │     │                   │       │                    │      │                           │  │                     │
///   │ └─┬──────────────────────┘       └─┬──────────┘           └─┬────────────────┘     └─┬─────────────────┘       └─┬──────────────────┘      └─┬─────────────────────────┘  └──┬──────────────────┘
///   │   │                                │                        │                        │                           │                           │                               │
///   │   │                                │                        │                        │                           │                           │                               │
///   │ ┌─▼───────────────────────────┐  ┌─▼───────────────────┐  ┌─▼───────────────────┐  ┌─▼──────────────────────┐  ┌─▼──────────────────────┐  ┌─▼──────────────────────┐     ┌──▼───────────────────┐
///   │ │                             │  │                     │  │                     │  │                        │  │                        │  │                        │     │                      │
///   │ │  ActorFactory::new_actor()  │  │  Actor::on_panic()  │  │  Actor::on_error()  │  │  Handler<M>::handle()  │  │  Handler<M>::handle()  │  │  Handler<M>::handle()  │     │  Actor::post_stop()  │
///   │ │                             │  │                     │  │                     │  │                        │  │                        │  │                        │     │                      │
///   │ │  Actor::pre_start()         │  └─┬───────────────────┘  └─┬───────────────────┘  └─┬──────────────────────┘  └─┬──────────────────────┘  └─┬──────────────────────┘     └──▲───────────────────┘
///   │ │                             │    │                        │                        │                           │                           │                               │
///   │ └─┬───────────────────────────┘    │                        │                        │                           │                           │                               │
///   │   │                                │                        │                        │                         ┌─▼─────────────────┐         │                               │
///   │   │                                │                        │                        └─────────────────────────►                   ◄─────────┘                               │
///   │   │                                │                        │                                                  │  is_stopped &&    │                                         │
///   │   │                                │                        │                                                  │                   │                                         │
///   │   │                                │                        │                                                  │  mailbox.empty()  │                                         │
///   │   │                                │                        │                                                  │                   │                                         │
///   │   │                                │                        │                                                  │  false      true  │                                         │
///   │   │                                │                        │                                                  │                   │                                         │
///   │   │                                │                        │                                                  └──┬────────────┬───┘                                         │
///   │   │                                │                        │                                                     │            │                                             │
///   └─◄─┴──────────────────────────────◄─┴──────────────────────◄─┴─────────────────────────────────────────────────────┘            └─────────────────────────────────────────────┘
///
/// ```
pub trait Actor: Send + Sync + UnwindSafe + Sized {
    /// executed whenever Actor receives a [SerializedMessage](../prelude/struct.SerializedMessage.html)
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    fn handle_serialized_message(
        &mut self,
        _msg: SerializedMessage,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        return Ok(ActorResult::Ok);
    }

    /// executed whenever a panic occurs within the actor
    ///
    /// determines actor behavior in case of a panic based on return value
    /// WARNING: returning `ActorResult::Restart` if `source == ActorPanicSource::Restart` can potentially result in an endless loop that will block the thread the actor is running on, until the restart was successful
    /// NOTE: if this function panics it will trigger a second time with `source == ActorPanicSource::OnPanic`, if another panic occurs in that case, the actor will be stopped immediately without finishing the mailbox!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn on_panic(&mut self, context: &ActorContext<Self>, source: ActorPanicSource) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.system.stop(Duration::from_millis(1000));
    ///         return Ok(ActorResult::Kill);
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         panic!("trigger on panic");
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn on_panic(&mut self, _context: &ActorContext<Self>, source: ActorPanicSource) -> Result<ActorResult, Box<dyn Error>> {
        return match source {

            ActorPanicSource::PreStart => {
                Ok(ActorResult::Ok)
            }
            ActorPanicSource::Message => {
                Ok(ActorResult::Restart)
            }
            ActorPanicSource::Restart => {
                Ok(ActorResult::Ok)
            }
            ActorPanicSource::OnPanic => {
                Ok(ActorResult::Kill)
            }
        }
    }

    /// executed whenever an error occurs within the actor
    ///
    /// determines actor behavior in case of an error based on return value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    /// use std::io::ErrorKind;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn on_error(&mut self, context: &ActorContext<Self>, err: Box<dyn Error>) -> ActorResult {
    ///         context.system.stop(Duration::from_millis(1000));
    ///         return ActorResult::Kill;
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         let error = std::io::Error::from_raw_os_error(22);
    ///         return Err(Box::new(error));
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn on_error(&mut self, _context: &ActorContext<Self>, err: Box<dyn Error>) -> ActorResult {
        error!("{:?}", err);
        return ActorResult::Ok;
    }

    /// executed before the first message is handled
    ///
    /// re-executed after actor restart before first message is handled
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::PreStart`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn pre_start(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.system.stop(Duration::from_millis(1000));
    ///         return Ok(ActorResult::Kill);
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         return Ok(ActorResult::Ok);
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn pre_start(&mut self, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        return Ok(ActorResult::Ok);
    }

    /// executed before the actor is restarted
    ///
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Restart`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn pre_restart(&mut self, context: &ActorContext<Self>) {
    ///         context.system.stop(Duration::from_millis(1000));
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         return Ok(ActorResult::Restart);
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn pre_restart(&mut self, _context: &ActorContext<Self>) {}

    /// executed after the last message is handled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn post_stop(&mut self, context: &ActorContext<Self>) {
    ///         context.system.stop(Duration::from_millis(1000));
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         return Ok(ActorResult::Stop);
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn post_stop(&mut self, _context: &ActorContext<Self>) {}

    /// executed when Actor handles internal ActorStopMessage
    ///
    /// If the return value is neither ActorResult::Stop nor ActorResult::StopImmediately the return value will internally be converted to ActorResult::Stop
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn on_actor_stop(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.system.stop(Duration::from_millis(1000));
    ///         return Ok(ActorResult::Kill);
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.actor_ref.stop()?;
    ///         return Ok(ActorResult::Ok);
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     std::process::exit(actor_system.await_shutdown());
    /// }
    /// ```
    fn on_actor_stop(&mut self, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        return Ok(ActorResult::Stop);
    }

    /// executed when Actor handles internal SystemStopMessage initiated by [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    ///
    /// Default behavior sends an `ActorStopMessage` to all actors which will trigger a clean shutdown
    /// panic triggers `self.on_panic()` with `source = ActorPanicSource::Message`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tyra::prelude::*;
    /// use std::error::Error;
    /// use std::time::Duration;
    ///
    /// struct TestActor {}
    /// impl TestActor {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl Actor for TestActor {
    ///     fn on_system_stop(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.system.stop_with_code(Duration::from_millis(1), 1337);
    ///         return Ok(ActorResult::Kill);
    ///     }
    /// }
    ///
    /// struct TestActorFactory {}
    /// impl TestActorFactory {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    /// impl ActorFactory<TestActor> for TestActorFactory {
    ///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
    ///         Ok(TestActor::new())
    ///     }
    /// }
    ///
    /// impl Handler<ActorInitMessage> for TestActor {
    ///     fn handle(&mut self, _msg: ActorInitMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
    ///         context.system.stop(Duration::from_millis(1000));
    ///         return Ok(ActorResult::Ok);
    ///     }
    /// }
    ///
    /// #[ntest::timeout(2000)]
    /// fn main() {
    ///     let actor_config = TyraConfig::new().unwrap();
    ///     let actor_system = ActorSystem::new(actor_config);
    ///     let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    ///     actor.send(ActorInitMessage::new()).unwrap();
    ///     let result = actor_system.await_shutdown();
    ///     assert_eq!(1337, result);
    /// }
    /// ```
    fn on_system_stop(&mut self, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let result = context.actor_ref.send(ActorStopMessage::new());
        if result.is_err() {
            debug!("")
        }
        return Ok(ActorResult::Ok);
    }
}
