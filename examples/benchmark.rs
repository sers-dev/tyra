#![allow(unused)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tyractorsaur::prelude::{
    ActorHandler, ActorSystem, ActorTrait, Context, Handler, MessageTrait, TyractorsaurConfig,
};

#[derive(Clone)]
struct MessageA {
    id: usize,
}

impl MessageTrait for MessageA {}

#[derive(Clone)]
struct Benchmark {
    total_msgs: usize,
    name: String,
    count: usize,
    start: Instant,
}

impl ActorTrait for Benchmark {}

impl Handler<MessageA> for Benchmark {
    fn handle(&mut self, msg: MessageA, context: &Context<Self>) {
        if self.count == 0 {
            println!("Sleep 3 now");
            sleep(Duration::from_secs((3) as u64));
            println!("Sleep 3 end");
            self.start = Instant::now();
        }
        self.count += 1;
        let wip_print = self.total_msgs / 10;
        if self.count % wip_print == 0 {
            println!("{} Counter: {}", self.name, self.count)
        }
        if self.count % self.total_msgs == 0 {
            let duration = self.start.elapsed();
            println!(
                "{} It took {:?} to process {} messages",
                self.name, duration, self.total_msgs
            );
        }
        if self.count == self.total_msgs {
            context.system.stop(Duration::from_secs(60));
        }
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let message_count = 10000000;

    let a = Benchmark {
        total_msgs: message_count as usize,
        name: (String::from("benchmark")),
        count: 0,
        start: Instant::now(),
    };
    let mut actor = actor_system.builder("benchmark-single-actor").build(a);
    println!("Actors have been created");
    let start = Instant::now();

    let id = 0;
    for i in 0..message_count {
        let msg = MessageA { id: i as usize };
        actor.send(msg);
    }
    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    exit(actor_system.await_shutdown());
}
