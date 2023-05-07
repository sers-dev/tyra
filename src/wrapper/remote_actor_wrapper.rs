use std::hash::{Hash, Hasher};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::actor::actor_address::ActorAddress;
use crate::message::actor_message::BaseActorMessage;
use crate::actor::actor_send_error::ActorSendError;
use crate::prelude::SerializedMessage;
use crate::system::system_state::SystemState;

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteActorWrapper {
    #[serde(skip)]
    system_state: Option<SystemState>,
}

impl RemoteActorWrapper
{
    pub fn new(
        system_state: SystemState,
    ) -> Self {
        let system_state = Some(system_state);
        return Self {
            system_state
        };
    }

    pub fn send<M>(&self, msg: M, address: &ActorAddress) -> Result<(), ActorSendError>
        where
            M: BaseActorMessage + 'static,
    {
        //todo this needs to forward to the NetWorker
        //the Networker should then forward this to the remote actor system
        //the remote actor system should then forward the message using system_state
        let serialized = bincode::serialize(&msg).unwrap();
        self.system_state.as_ref().unwrap().send_to_address(address, SerializedMessage::new(serialized));
        return Ok(());
    }

    pub fn send_timeout<M>(&self, _msg: M, _timeout: Duration, _address: ActorAddress) -> Result<(), ActorSendError>
        where
            M: BaseActorMessage + 'static,
    {
        return Ok(());
    }

    pub fn send_after<M>(&self, _msg: M, _delay: Duration, _address: ActorAddress) -> Result<(), ActorSendError>
        where
            M: BaseActorMessage + 'static,
    {
        return Ok(());
    }

    pub fn stop(&self) -> Result<(), ActorSendError> {
        return Ok(());
    }

    pub fn sleep(&self, _duration: Duration, _address: ActorAddress) -> Result<(), ActorSendError> {
        return Ok(());
    }

    pub fn get_mailbox_size(&self) -> usize {
        return 0;
    }

    pub fn is_mailbox_stopped(&self) -> bool {
        return true;
    }

    pub fn is_stopped(&self) -> bool {
        return true;
    }

    pub fn wait_for_stop(&self) {
        return;
    }
}

impl Hash for RemoteActorWrapper
{
    fn hash<H: Hasher>(&self, _state: &mut H) {
        return;
    }
}