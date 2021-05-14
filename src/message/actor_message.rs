/// Core trait to define Messages
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyractorsaur::prelude::ActorMessage;
///
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
/// ```
pub trait ActorMessage: Send + Sync {}
