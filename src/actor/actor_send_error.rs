use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ActorSendError {
    /// Triggered by [ActorWrapper.send_timeout](../prelude/struct.ActorWrapper.html#method.send_timout) if message can't be delivered without user defined timeout
    #[error("Message could not be delivered in time")]
    TimeoutError,

    /// Triggered by [ActorWrapper.send](../prelude/struct.ActorWrapper.html#method.send) && [ActorWrapper.send_timeout](../prelude/struct.ActorWrapper.html#method.send_timout) when a message is sent to a stopped Actor
    #[error("Message could not be delivered")]
    AlreadyStoppedError,

    /// Triggered by [ActorWrapper](../prelude/struct.ActorWrapper.html) if a message can't be send to remote Actors
    #[error("Message can't be delivered to remote Actor")]
    NotAllowedForRemoteActorError
}
