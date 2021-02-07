use crate::actor::{ActorTrait, Handler};
use std::sync::{Arc, RwLock};
use crate::message::MessageTrait;
use crossbeam_channel::{Sender, Receiver};

#[derive(Clone)]
pub struct ActorRef<A>
    where
        A: ActorTrait,
{
    actor: Arc<RwLock<A>>,
    mailbox_in: Sender<String>,
    mailbox_out: Receiver<String>

}

impl<A> ActorRef<A>
    where
        A: ActorTrait,
{
    pub fn new(actor: Arc<RwLock<A>>, sender: Sender<String>, receiver: Receiver<String>) -> Self {

        Self {
            actor,
            mailbox_in: sender,
            mailbox_out: receiver,
        }
    }
    pub fn send<M>(&mut self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait
    {
        let mut actor = self.actor.write().unwrap();
        actor.handle(msg);
        println!("AAAAAAAAAAAAAAA")
    }
}