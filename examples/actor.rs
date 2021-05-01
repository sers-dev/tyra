use std::process::exit;
use std::time::Duration;
use tyractorsaur::prelude::{ActorSystem, ActorTrait, Context, Handler, MessageTrait, TyractorsaurConfig, ActorProps};

struct MessageA {
    text: String,
}

struct MessageB {
    text: String,
}

impl MessageTrait for MessageA {}

impl MessageTrait for MessageB {}

struct MessageUnsupported {}

impl MessageTrait for MessageUnsupported {}

struct HelloWorld {
    text: String,
    count: usize,
}

impl ActorTrait for HelloWorld {}

struct HelloWorldProps {
    text: String,
    count: usize,
}

impl ActorProps<HelloWorld> for HelloWorldProps {
    fn new_actor(&self, _context: Context<HelloWorld>) -> HelloWorld {
        HelloWorld {
            count: self.count,
            text: self.text.clone()
        }
    }
}
impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, msg: MessageA, _context: &Context<Self>) {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count += 1;
        println!("AAAA: {} Count: {}", text, self.count)
    }
}

impl Handler<MessageB> for HelloWorld {
    fn handle(&mut self, msg: MessageB, _context: &Context<Self>) {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count -= 1;
        println!("BBBB: {} Count: {}", text, self.count)
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.add_pool("aye");
    actor_system.add_pool("aye2");

    let hw = HelloWorldProps {
        text: String::from("sers"),
        count: 0,
    };
    let x = actor_system
        .builder("hello-world")
        .set_mailbox_size(7)
        .set_pool_name("aye")
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

    actor_system.stop(Duration::from_secs(1));
    exit(actor_system.await_shutdown());
}
