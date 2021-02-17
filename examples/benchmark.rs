#![allow(unused)]

use tractor::prelude::{TractorConfig, ActorSystem, ActorTrait, Handler, MessageTrait, ActorRef};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::sync::Arc;
use std::any::{TypeId, Any};
use std::collections::HashMap;

#[derive(Clone)]
struct MessageA {
    text: String

}

impl MessageTrait for MessageA {}

#[derive(Clone)]
struct Benchmark {
    total_msgs: usize,
    name: String,
    count: usize,
    start: Instant
}

impl ActorTrait for Benchmark {
}

impl Handler<MessageA> for Benchmark {
    fn handle(&mut self, msg: MessageA) {
        if self.count == 0 {
            println!("Sleep 25 now");
            sleep(Duration::from_secs((25) as u64));
            println!("Sleep 25 end");
            self.start = Instant::now();
        }
        self.count += 1;
        let wip_print = self.total_msgs / 10;
        if self.count % wip_print == 0 {
            println!("B-{} Counter: {}", self.name, self.count)
        }
        if self.count % self.total_msgs == 0 {
            let duration = self.start.elapsed();
            println!("B-{} It took {:?} to process {} messages", self.name, duration, self.total_msgs);
        }
    }
}


fn main() {
    let actor_config = TractorConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let actor_count :i32 = 1;
    let message_count = 10000000 / actor_count;


    let pool_size = 32;
    actor_system.add_pool("aye", pool_size as usize);

    let mut actors :HashMap<i32, ActorRef<Benchmark>> = HashMap::new();
    for i in 0..actor_count {
        let a = Benchmark{ total_msgs: message_count as usize, name: (i.to_string()), count: 0, start: Instant::now()};
        let mut r = actor_system.builder("hello-world").set_mailbox_size(7).set_pool("aye").build(a);
        actors.insert(i, r);
    }
    println!("Actors have been created");
    let start = Instant::now();

    for i in 0..message_count {
        for j in 0..actor_count {
            let actor_id = j;
            let mut a = actors.get_mut(&actor_id).unwrap();
            a.send(MessageA { text: String::from("sers+1") });
        }
    }
    println!("Messages have been sent");

    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    actor_system.await_shutdown()

}