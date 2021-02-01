use crate::message::MessageTrait;
use serde::{Deserialize, Serialize};
use crate::context::Context;

pub trait ActorTrait: Send + Sync {
}

pub trait Handler<M>
where
    Self: ActorTrait,
    M: MessageTrait
{
    fn handle(&self, msg: M);
}

struct Envelope {
    content: dyn MessageTrait
}

struct ActorRef {
    actor: Box<dyn ActorTrait>,
    context: Context
}

impl ActorRef {
   fn tell<M>(&self, msg: M) {

   }

    fn is_running(&self) {
        println!("ACTOR-{}", "self.actor");

    }

}
/////

//////

pub struct HelloWorld {
    pub text: String
}

impl ActorTrait for HelloWorld {
    //fn handle(&self, msg: impl MessageTrait) {
//
    //}
}

impl Handler<Message> for HelloWorld {
    fn handle(&self, msg: Message) {
        let text :String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        println!("{}", text)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub text: String

}

impl Message {

}

impl MessageTrait for Message {}