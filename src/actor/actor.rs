use std::panic::UnwindSafe;
use crate::message::serialized_message::SerializedMessage;

pub trait Actor: Send + Sync + UnwindSafe {
    fn pre_start(&mut self) {}
    fn post_stop(&mut self) {}
    fn on_actor_stop(&mut self) {}
    fn on_system_stop(&mut self) {}
    fn handle_serialized_message(&self, _msg: SerializedMessage) {}
}

