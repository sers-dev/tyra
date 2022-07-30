use crate::actor::base_actor::BaseActor;
use crate::actor::context::ActorContext;
use std::panic::UnwindSafe;

/// [Actor] can only be created from a Factory
///
/// This factory approach is necessary because of the restart behavior.
/// Without this factory we'd need to keep a `.clone()` of the initial Actor, which would force all Actor implementations to implement `Clone`.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::{BaseActor, SerializedMessage, ActorFactory, ActorContext, Handler};
///
/// struct TestActor {}
///
/// impl BaseActor for TestActor {}
///
/// impl Handler<SerializedMessage> for TestActor {
///     fn handle(&mut self, _msg: SerializedMessage, _context: &ActorContext<Self>) {
///     }
/// }
///
/// struct TestFactory {}
///
/// impl ActorFactory<TestActor> for TestFactory {
///     fn new_actor(&self, _context: ActorContext<TestActor>) -> TestActor {
///         TestActor {}
///     }
/// }
/// ```
pub trait ActorFactory<A>
where
    A: BaseActor + UnwindSafe + 'static,
{
    /// internally used to create the Actual Actor
    ///
    /// `ActorContext<A>` is injected and can optionally be stored within the actor itself.
    /// It can then be used to define clean a behavior for a clean [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    /// through [Actor.on_system_stop]
    fn new_actor(&self, context: ActorContext<A>) -> A;
}
