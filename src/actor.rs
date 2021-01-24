use crate::message::MessageTrait;

pub trait ActorTrait: Sized {}

pub trait Handler<M>
where
    Self: ActorTrait,
    M: MessageTrait
{
    fn handle(&mut self, msg: M);
}

