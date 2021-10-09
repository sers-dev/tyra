/// Core trait to define Messages
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::ActorMessage;
///
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
/// ```
pub trait ActorMessage: Send + Sync {}
