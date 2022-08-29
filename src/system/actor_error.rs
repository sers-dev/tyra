use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActorError {
    /// Triggered by [ActorBuilder.spawn](../prelude/struct.ActorBuilder.html#method.spawn) if panic happens within [new_actor](../prelude/trait.ActorFactory.html#method.new_actor)
    #[error("Actor could not be started")]
    InitError,

    /// Triggered by [ActorBuilder.spawn](../prelude/struct.ActorBuilder.html#method.spawn) if the actor already exists, but the type is not equal to the type of the ActorBuilder<A>
    #[error("Actor exists, but is not of the expected Type")]
    InvalidActorTypeError,

    /// Triggered by [ActorBuilder.spawn](../prelude/struct.ActorBuilder.html#method.spawn) if the actor can't be spawned, because the thread-pool already reached the configured actor limit
    #[error("Actor could not be started, because thread-pool is full")]
    ThreadPoolHasTooManyActorsError,

    /// Triggered by [ActorBuilder.spawn](../prelude/struct.ActorBuilder.html#method.spawn) if the actor can't be spawned, because the thread-pool does not exist
    #[error("Actor could not be started, because thread-pool does not exist")]
    ThreadPoolDoesNotExistError,
}