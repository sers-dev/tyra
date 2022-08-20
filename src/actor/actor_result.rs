#[derive(PartialEq)]
/// ActorResults that determine how the actor should proceed
pub enum ActorResult {
    /// continue processing messages
    Ok,
    /// lock mailbox, stop after mailbox is empty
    Stop,
    /// restart immediately, continue processing messages afterwards
    Restart,
    /// stop immediately and ignore any remaining messages in the mailbox
    Kill,
}