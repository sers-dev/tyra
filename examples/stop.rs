#![allow(unused)]

use std::any::{Any, TypeId};
use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{ActorHandlerTrait, ActorSystem, ActorTrait, Context, Handler, MessageTrait, TyractorsaurConfig, ActorProps};

#[derive(Clone)]
struct TestMsg {}

impl MessageTrait for TestMsg {}

#[derive(Clone)]
struct StopActor {}

impl ActorTrait for StopActor {
    fn pre_start(&mut self, context: &Context<Self>) {
        println!("PRE START")
    }
    fn post_stop(&mut self, context: &Context<Self>) {
        context.system.stop(Duration::from_secs(1));
        println!("POST STOP");
    }
}

impl Handler<TestMsg> for StopActor {
    fn handle(&mut self, msg: TestMsg, context: &Context<Self>) {
        context.actor_ref.send(TestMsg {});
        println!("Message received!");
        sleep(Duration::from_millis(100));
    }
}

struct StopActorProps {}

impl ActorProps<StopActor> for StopActorProps {
    fn new_actor(&self) -> StopActor {
        StopActor{}
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = StopActorProps {};
    let mut x = actor_system
        .builder("hello-world")
        .set_mailbox_size(7)
        .build(hw);
    // this is obviously handled, because it's the actor is still running
    x.send(TestMsg {});
    sleep(Duration::from_millis(700));

    x.stop();
    // this is still handled, because the actor has not handled the stop Message yet
    x.send(TestMsg {});
    sleep(Duration::from_millis(200));
    // this is no longer handled, because the actor has stopped by now
    x.send(TestMsg {});

    let result = actor_system.await_shutdown();

    exit(result);
}
