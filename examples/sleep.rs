#![allow(unused)]

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{
    ActorRefTrait, ActorSystem, ActorTrait, Handler, MessageTrait, TyractorsaurConfig,
};

#[derive(Clone)]
struct SleepMsg {
    text: String,
}

impl MessageTrait for SleepMsg {}

#[derive(Clone)]
struct SleepActor {
    text: String,
    counter: usize,
}

impl ActorTrait for SleepActor {}

impl Handler<SleepMsg> for SleepActor {
    fn handle(&mut self, msg: SleepMsg) {
        let text: String = [self.text.clone(), String::from(msg.text.clone())].join(" -> ");
        self.counter += 1;
        //if self.counter == 1 {
            sleep(Duration::from_secs(3));
        //}
        //if self.counter % 1000000 == 0 {
            println!("Received SERS: {}", self.counter);
        //}
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = SleepActor {
        text: String::from("sers"),
        counter: 0,
    };
    let mut x = actor_system
        .builder("hello-world")
        .set_mailbox_unbounded()
        .build(hw);
    x.send(SleepMsg {
        text: String::from("sers+1"),
    });

    sleep(Duration::from_secs(1));

    x.send(SleepMsg {
        text: String::from("sers+2"),
    });

    sleep(Duration::from_secs(1));

    x.send(SleepMsg {
        text: String::from("sers+2"),
    });
    //loop {
    //    //sleep(Duration::from_micros(1));
    //    x.send(SleepMsg {
    //        text: String::from("sers+2"),
    //    });
    //    //println!("SEND NOW");
    //    //sleep(Duration::from_micros(400));
    //}


    actor_system.await_shutdown()
}