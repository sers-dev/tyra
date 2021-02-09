use crate::actor::{ActorTrait, Handler};
use std::sync::{Arc, RwLock};
use crate::message::MessageTrait;
use crossbeam_channel::{Sender, Receiver, unbounded};

#[derive(Clone)]
pub struct ActorRef<A>
    where
        A: ActorTrait,
{
    actor: Arc<RwLock<A>>,
    mailbox_in: Sender<Arc<dyn MessageTrait>>,
    mailbox_out: Receiver<Arc<dyn MessageTrait>>,
}

impl<A> ActorRef<A>
    where
        A: ActorTrait,
{
    pub fn new(actor: Arc<RwLock<A>>, sender: Sender<Arc<dyn MessageTrait>>, receiver: Receiver<Arc<dyn MessageTrait>>) -> Self {
        Self {
            actor,
            mailbox_in: sender,
            mailbox_out: receiver
        }
    }
    pub fn send<M>(&mut self, msg: M)
        where
            A: Handler<M>,
            M: MessageTrait + Clone + 'static
    {
        let abcd = msg.clone();
        self.mailbox_in.send(Arc::new(abcd));
        let mut actor = self.actor.write().unwrap();
        actor.handle(msg);
        println!("AAAAAAAAAAAAAAA")
    }
}