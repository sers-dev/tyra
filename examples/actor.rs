#![allow(unused)]

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{ActorSystem, ActorTrait, Handler, MessageTrait, TyractorsaurConfig};

#[derive(Clone)]
struct MessageA {
    text: String,
}

#[derive(Clone)]
struct MessageB {
    text: String,
}

impl MessageTrait for MessageA {}

impl MessageTrait for MessageB {}

#[derive(Clone)]
struct MessageUnsupported {
    text: String,
}

impl MessageTrait for MessageUnsupported {}

#[derive(Clone)]
struct HelloWorld {
    text: String,
    count: usize,
}

impl ActorTrait for HelloWorld {}

impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, msg: MessageA) {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count += 1;
        println!("AAAA: {} Count: {}", "text", self.count)
    }
}

impl Handler<MessageB> for HelloWorld {
    fn handle(&mut self, msg: MessageB) {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count -= 1;
        println!("BBBB: {} Count: {}", "text", self.count)
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.add_pool("aye", 7);

    let hw = HelloWorld {
        text: String::from("sers"),
        count: 0,
    };
    let mut x = actor_system
        .builder("hello-world")
        .set_mailbox_size(7)
        .set_pool("aye")
        .build(hw);
    x.send(MessageA {
        text: String::from("sers+1"),
    });
    x.send(MessageA {
        text: String::from("sers+2"),
    });
    x.send(MessageB {
        text: String::from("sers-1"),
    });
    x.send(MessageA {
        text: String::from("sers+3"),
    });
    x.send(MessageA {
        text: String::from("sers+4"),
    });
    x.send(MessageA {
        text: String::from("sers+5"),
    });

    //x.send(MessageUnsupported{text: String::from("sers")});

    actor_system.await_shutdown()
}
