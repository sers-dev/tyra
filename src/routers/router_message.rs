use crate::message::actor_message::ActorMessage;

/// Wraps any [ActorMessage](../prelude/trait.ActorMessage.html) to be sent to a Router
pub struct RouterMessage<M>
where
    M: ActorMessage + 'static,
{
    pub msg: M,
}

impl<M> ActorMessage for RouterMessage<M>
where
    M: ActorMessage + 'static,
{
    fn get_id(&self) -> usize {
        return self.msg.get_id();
    }
}

impl<M> RouterMessage<M>
where
    M: ActorMessage + 'static,
{
    pub fn new(msg: M) -> Self {
        Self { msg }
    }
}
