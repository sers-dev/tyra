use tractor::prelude::{TractorConfig, ActorSystem, ActorTrait, Handler, MessageTrait};
use std::time::Duration;
use std::thread::sleep;

struct Message {
    text: String

}



impl MessageTrait for Message {

}


struct MessageUnsupported {
    text: String

}

impl MessageTrait for MessageUnsupported {}

struct HelloWorld {
    text: String
}

impl ActorTrait for HelloWorld {
}

impl Handler<Message> for HelloWorld {
    fn handle(&self, msg: Message) {
        let text :String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        println!("{}", text)
    }
}

fn main() {
    let actor_config = TractorConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    sleep(Duration::from_secs(3));

    actor_system.add_pool("aye", 7);
    actor_system.add_pool("aye", 100);

    let x = actor_system.spawn("sers-actor").set_mailbox_size(7).set_pool("na-geil").build(HelloWorld{ text: String::from("sers")});
    x.send(Message{text: String::from("sers")});
    //x.send(MessageUnsupported{text: String::from("sers")});


    actor_system.await_shutdown()

}