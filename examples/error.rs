#![allow(unused)]

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{
    ActorRefTrait, ActorSystem, ActorTrait, Handler, MessageTrait, TyractorsaurConfig,
};

#[derive(Clone)]
struct ErrMsg {
    text: String,
}

impl MessageTrait for ErrMsg {}

#[derive(Clone)]
struct ErrActor {
    text: String,
    counter: usize,
}

impl ActorTrait for ErrActor {}

impl Handler<ErrMsg> for ErrActor {
    fn handle(&mut self, msg: ErrMsg) {
        let text: String = [self.text.clone(), String::from(msg.text.clone())].join(" -> ");
        self.counter += 1;
        if msg.text == "sers+1" {
            panic!("ficl");
        }
        println!("Received SERS: {}", self.counter);
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = ErrActor {
        text: String::from("sers"),
        counter: 0,
    };
    let mut x = actor_system
        .builder("hello-world")
        .set_mailbox_size(7)
        .build(hw);
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

    actor_system.await_shutdown()
}
