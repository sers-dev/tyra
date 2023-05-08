use crate::actor::actor_address::ActorAddress;
use crate::actor::actor_send_error::ActorSendError;
use crate::message::actor_message::BaseActorMessage;
use crate::system::system_state::SystemState;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use crate::prelude::ActorSendError::NotAllowedForRemoteActorError;

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteActorWrapper {
    #[serde(skip)]
    system_state: Option<SystemState>,
}

impl RemoteActorWrapper {
    pub fn new(system_state: SystemState) -> Self {
        let system_state = Some(system_state);
        return Self { system_state };
    }

    pub fn send<M>(&self, msg: M, address: &ActorAddress) -> Result<(), ActorSendError>
    where
        M: BaseActorMessage + 'static,
    {
        let serialized = bincode::serialize(&msg).unwrap();
        self.system_state
            .as_ref()
            .unwrap()
            .send_to_address(address, serialized);
        return Ok(());
    }

    pub fn send_timeout<M>(
        &self,
        msg: M,
        _timeout: Duration,
        address: ActorAddress,
    ) -> Result<(), ActorSendError>
    where
        M: BaseActorMessage + 'static,
    {
        // since we don't work with the mailbox directly for remote actors, we can't send messages with timeout
        // instead of giving an error, we'll simply send the message and ignore the timeout
        return self.send(msg, &address);
    }

    pub fn send_after<M>(
        &self,
        _msg: M,
        _delay: Duration,
        _address: ActorAddress,
    ) -> Result<(), ActorSendError>
    where
        M: BaseActorMessage + 'static,
    {
        //send serialized version of DelayedMessage to system_state
        //destination address in SerializedMessage needs to be the delay-router on the remote system
        return Ok(());
    }

    pub fn stop(&self) -> Result<(), ActorSendError> {
        return Err(NotAllowedForRemoteActorError);
    }

    pub fn sleep(&self, _duration: Duration, _address: ActorAddress) -> Result<(), ActorSendError> {
        return Err(NotAllowedForRemoteActorError);
    }

    pub fn get_mailbox_size(&self) -> usize {
        return 0;
    }

    pub fn is_mailbox_stopped(&self) -> bool {
        return false;
    }

    pub fn is_stopped(&self) -> bool {
        return false;
    }

    pub fn wait_for_stop(&self) {
        return;
    }
}

impl Hash for RemoteActorWrapper {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        return;
    }
}
