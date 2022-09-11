use std::error::Error;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use tyra::prelude::{Actor, ActorContext, ActorError, ActorFactory, ActorMessage, ActorResult, ActorSystem, Handler, TyraConfig};

struct MessageA {
    text: String,
}

struct MessageB {
    text: String,
}

impl ActorMessage for MessageA {}

impl ActorMessage for MessageB {}

struct MessageUnsupported {}

impl ActorMessage for MessageUnsupported {}

struct HelloWorld {
    text: String,
    count: usize,
}

impl Actor for HelloWorld {
    fn on_error(&mut self, _context: &ActorContext<Self>, err: Box<dyn Error>) -> ActorResult {
        println!("{:?}", err);
        if err.is::<ActorError>() {
            return ActorResult::Ok;
        }
        if err.is::<std::io::Error>() {
            return ActorResult::Stop;
        }
        return ActorResult::Kill;
    }
}

struct HelloWorldFactory {
    text: String,
    count: usize,
}

impl ActorFactory<HelloWorld> for HelloWorldFactory {
    fn new_actor(&mut self, _context: ActorContext<HelloWorld>) -> Result<HelloWorld, Box<dyn Error>> {
        Ok(HelloWorld {
            count: self.count,
            text: self.text.clone(),
        })
    }
}
impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, msg: MessageA, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count += 1;
        println!("AAAA: {} Count: {}", text, self.count);
        return Ok(ActorResult::Ok);
    }
}

impl Handler<MessageB> for HelloWorld {
    fn handle(&mut self, msg: MessageB, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        let text: String = [self.text.clone(), String::from(msg.text)].join(" -> ");
        self.count -= 1;
        println!("BBBB: {} Count: {}", text, self.count);
        return Ok(ActorResult::Ok);
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.add_pool("aye");
    actor_system.add_pool("aye2");

    let hw = HelloWorldFactory {
        text: String::from("sers"),
        count: 0,
    };
    let x = actor_system
        .builder()
        .set_mailbox_size(7)
        .set_pool_name("aye")
        .spawn("hello-world", hw)
        .unwrap();
    let _ = x.send(MessageA {
        text: String::from("sers+1"),
    });
    let _ = x.send(MessageA {
        text: String::from("sers+2"),
    });
    let _ = x.send(MessageB {
        text: String::from("sers-1"),
    });
    let _ = x.send(MessageA {
        text: String::from("sers+3"),
    });
    let _ = x.send(MessageA {
        text: String::from("sers+4"),
    });
    let _ = x.send(MessageA {
        text: String::from("sers+5"),
    });

    //x.send(MessageUnsupported{text: String::from("sers")});

    sleep(Duration::from_millis(125));
    actor_system.stop(Duration::from_secs(1));
    exit(actor_system.await_shutdown());
}
