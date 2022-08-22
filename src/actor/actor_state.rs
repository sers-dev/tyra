use std::time::Duration;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActorState {
    Running,
    Inactive,
    Stopped,
    Sleeping(Duration),
}
