use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// This trait is used internally by the `ActorSystem` and builds the base for all messaging
/// It's automatically implemented by the `ActorMessage` trait that should be used
///
/// It is used by Messages defined in the system
/// All messages that use this trait directly should also implement a dynamic `Handler<M>` that applies to any `Actor`
pub trait BaseActorMessage: Send + Sync + Hash {
}

/// This trait is used by Messages defined by the system
/// All messages that use this trait should also implement a dynamic `Handler<M>` that applies to any `Actor`
pub trait DefaultActorMessage: Send + Sync + Hash {
}

impl<A> BaseActorMessage for A
where
    A: DefaultActorMessage
{}

/// Core trait to define Messages
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use tyra::prelude::ActorMessage;
///
/// #[derive(Hash)]
/// struct FooBar {}
/// impl ActorMessage for FooBar {}
/// ```
pub trait ActorMessage: Send + Sync + Hash {
    /// returns the message hash
    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return hasher.finish();
    }
}

/// this should be `BaseActorMessage` but it's currently not possible because of https://github.com/rust-lang/rust/issues/20400
impl<A> DefaultActorMessage for A
    where
        A: ActorMessage
{}