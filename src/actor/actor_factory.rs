use crate::actor::context::ActorContext;
use crate::prelude::Actor;
use std::error::Error;
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
/// use std::error::Error;
/// use tyra::prelude::*;
///
/// struct TestActor {}
///
/// impl Actor for TestActor {}
///
/// struct TestFactory {}
///
/// impl ActorFactory<TestActor> for TestFactory {
///     fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
///         Ok(TestActor {})
///     }
/// }
/// ```
pub trait ActorFactory<A>: UnwindSafe
where
    A: Actor + 'static,
{
    /// internally used to create the Actual Actor
    ///
    /// `ActorContext<A>` is injected and can optionally be stored within the actor itself.
    /// It can then be used to define clean a behavior for a clean [ActorSystem.stop](../prelude/struct.ActorSystem.html#method.stop)
    /// through [Actor.on_system_stop]
    fn new_actor(&mut self, _context: ActorContext<A>) -> Result<A, Box<dyn Error>>;
}
