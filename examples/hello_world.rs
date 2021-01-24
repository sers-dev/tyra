use tractor::prelude::{ActorTrait, Handler, MessageTrait};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Message {
    text: String

}

impl Message {

}

impl MessageTrait for Message {}

struct HelloWorld {
    text: String
}

impl ActorTrait for HelloWorld {
}

impl Handler<Message> for HelloWorld {
    fn handle(&mut self, msg: Message) {
        let text :String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        println!("{}", text)
    }
}

pub fn main() {
    let mut foo = HelloWorld{
        text: String::from("Sers")
    };
    let far = Message{
        text: String::from("Bye")
    };

    foo.handle(far);

}