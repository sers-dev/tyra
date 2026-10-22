use std::error::Error;
use std::process::exit;
use std::time::{Duration, Instant};
use tyra::prelude::*;

#[derive(ActorMessage)]
struct MessageA {}

#[derive(Actor, ActorFactory)]
struct Benchmark {
    total_msgs: usize,
    name: String,
    #[tyra(default = "0")]
    count: usize,
    #[tyra(default = "Instant::now()")]
    start: Instant,
}

impl Handler<MessageA> for Benchmark {
    fn handle(
        &mut self,
        _msg: MessageA,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if self.count == 0 {
            println!("START PROCESSING!");
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
        Ok(ActorResult::Ok)
    }
}

fn main() {
    let mut actor_config = TyraConfig::new().unwrap();
    actor_config
        .thread_pool
        .config
        .insert(String::from("default"), ThreadPoolConfig::new(1, 1, 1, 1.0));
    let actor_system = ActorSystem::new(actor_config);

    let message_count = 10000000;

    let actor = actor_system
        .builder()
        .spawn(
            "benchmark-single-actor",
            Benchmark::factory(message_count as usize, String::from("benchmark")),
        )
        .unwrap();
    println!("Actors have been created");
    let start = Instant::now();

    actor.sleep(Duration::from_secs(3)).unwrap();
    for _i in 0..message_count {
        let msg = MessageA {};
        actor.send(msg).unwrap();
    }
    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    exit(actor_system.await_shutdown());
}
