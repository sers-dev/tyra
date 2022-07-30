use std::process::exit;
use std::time::Duration;
use tyra::prelude::{Actor, ActorFactory, ActorMessage, ActorSystem, ActorContext, Handler, TyraConfig, ActorMessageDeserializer};

#[derive(Clone)]
struct ErrMsg {
    text: String,
}

impl ActorMessage for ErrMsg {}

#[derive(Clone)]
struct ErrActor {
    text: String,
    counter: usize,
}

impl ActorMessageDeserializer for ErrActor {}

impl Handler<ErrMsg> for ErrActor {
    fn handle(&mut self, msg: ErrMsg, _context: &ActorContext<Self>) {
        self.counter += 1;
        if msg.text == "sers+1" {
            panic!("ficl");
        }
        println!("Received SERS: {}", self.counter);
    }
}

struct ErrActorFactory {
    text: String,
    counter: usize,
}

impl ActorFactory<ErrActor> for ErrActorFactory {
    fn new_actor(&self, _context: ActorContext<ErrActor>) -> ErrActor {
        ErrActor {
            text: self.text.clone(),
            counter: self.counter,
        }
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = ErrActorFactory {
        text: String::from("sers"),
        counter: 0,
    };
    let x = actor_system
        .builder()
        .set_mailbox_size(7)
        .spawn("hello-world", hw).unwrap();
    x.send(ErrMsg {
        text: String::from("sers+1"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });
    x.send(ErrMsg {
        text: String::from("sers+1"),
    });
    x.send(ErrMsg {
        text: String::from("sers+2"),
    });

    actor_system.stop(Duration::from_secs(5));
    exit(actor_system.await_shutdown())
}
