use tractor::prelude::{TractorConfig, ActorSystem, ActorTrait, Handler, MessageTrait};
use std::time::Duration;
use std::thread::sleep;

struct MessageA {
    text: String

}

struct MessageB {
    text: String
}

impl MessageTrait for MessageA {

}

impl MessageTrait for MessageB {

}

struct MessageUnsupported {
    text: String

}

impl MessageTrait for MessageUnsupported {}

struct HelloWorld {
    text: String,
    count: usize,
}

impl ActorTrait for HelloWorld {
}

impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, msg: MessageA) {
        let text :String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count += 1;
        println!("AAAA: {} Count: {}", text, self.count)
    }
}

impl Handler<MessageB> for HelloWorld {
    fn handle(&mut self, msg: MessageB) {
        let text :String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count -= 1;
        println!("BBBB: {} Count: {}", text, self.count)
    }
}



fn main() {
    let actor_config = TractorConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.add_pool("aye", 7);

    let hw = HelloWorld{ text: String::from("sers"), count: 0};

    let mut x = actor_system.spawn("sers-actor").set_mailbox_size(7).set_pool("default").build(hw);
    x.send(MessageA {text: String::from("sers+1")});
    x.send(MessageA {text: String::from("sers+2")});
    x.send(MessageB {text: String::from("sers-1")});
    x.send(MessageA {text: String::from("sers+3")});
    x.send(MessageA {text: String::from("sers+4")});
    x.send(MessageA {text: String::from("sers+5")});

    //x.send(MessageUnsupported{text: String::from("sers")});


    actor_system.await_shutdown()

}