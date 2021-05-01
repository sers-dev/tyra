#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MessageType {
    Unknown,
    ActorStopMessage,
    SystemStopMessage,
    RemoteMessage,
}
