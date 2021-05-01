#![allow(unused)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tyractorsaur::prelude::{ActorSystem, ActorTrait, Context, Handler, MessageTrait, TyractorsaurConfig, ActorProps, ActorRef, SerializedMessage};

struct MessageA {
    id: usize,
}

impl MessageTrait for MessageA {}


struct MessageB {
    id: usize,
    rea: ActorRef<Benchmark>
}

impl MessageTrait for MessageB {}


struct Benchmark {
    ctx: Context<Self>,
    total_msgs: usize,
    name: String,
    count: usize,
    start: Instant,
}

struct BenchmarkProps {
    total_msgs: usize,
    name: String,
}

impl ActorProps<Benchmark> for BenchmarkProps {
    fn new_actor(&self, context: Context<Benchmark>) -> Benchmark {
        Benchmark::new(self.total_msgs, self.name.clone(), context)
    }
}

impl Benchmark {
    pub fn new(total_msgs: usize, name: String, context: Context<Self>) -> Self {
        Self {
            ctx: context,
            total_msgs,
            name,
            count: 0,
            start: Instant::now(),
        }
    }
}

impl ActorTrait for Benchmark {
    fn pre_start(&mut self) {}

    fn post_stop(&mut self) {}

    fn on_actor_stop(&mut self) {}

    fn on_system_stop(&mut self) {
        self.ctx.actor_ref.stop();
    }

    fn handle_serialized_message(&self, msg: SerializedMessage) {

    }
}

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

    let mut actor = actor_system.builder("benchmark-single-actor").build(BenchmarkProps {
        name: String::from("benchmark"),
        total_msgs: message_count as usize,
    });
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
