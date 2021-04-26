#![allow(unused)]

use std::any::{Any, TypeId};
use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{ActorHandlerTrait, ActorSystem, ActorTrait, Context, Handler, MessageTrait, TyractorsaurConfig, ActorProps};

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
    fn handle(&mut self, msg: SleepMsg, context: &Context<Self>) {
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

struct SleepActorProps {
    text: String,
    counter: usize,
}

impl ActorProps<SleepActor> for SleepActorProps {
    fn new_actor(&self, context: Context<SleepActor>) -> SleepActor {
        SleepActor{
            counter: self.counter,
            text: self.text.clone(),
        }
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = SleepActorProps {
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

    actor_system.stop(Duration::from_secs(10));
    exit(actor_system.await_shutdown());
}
