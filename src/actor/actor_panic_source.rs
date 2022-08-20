#[derive(PartialEq)]
/// Information provided to [on_panic](../prelude/trait.Actor.html#method.on_panic) to distinguish cause of panic
pub enum ActorPanicSource {
    /// Triggered if panic happens within [pre_start](../prelude/trait.Actor.html#method.pre_start)
    PreStart,
    /// Triggered if panic happens within [any implemented Handler<M>](../prelude/trait.Handler.html#method.handle)
    Message,
    /// Triggered if panic happens within [new_actor](../prelude/trait.ActorFactory.html#method.new_actor)
    Restart,
    /// Triggered if panic happens within [on_panic](../prelude/trait.Actor.html#method.on_panic)
    /// Only triggered once, if panic-loop occurs, then the actor will be instantly stopped
    OnPanic,
}