#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActorState {
    Running,
    Sleeping,
    Stopped,
}
