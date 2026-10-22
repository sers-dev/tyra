use std::error::Error;
use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tyra::prelude::*;
use tyra::router::{AddActorMessage, BulkRouterMessage, RoundRobinRouter};

#[derive(ActorMessage)]
struct MessageA {}

#[derive(ActorMessage)]
struct Finish {}

#[derive(ActorMessage)]
struct Start {}

#[derive(Actor, ActorFactory)]
struct Benchmark {
    aggregator: ActorWrapper<Aggregator>,
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
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if self.count == 0 {
            sleep(Duration::from_secs((3) as u64));
            self.start = Instant::now();
        }
        self.count += 1;
        if self.count % self.total_msgs == 0 {
            let duration = self.start.elapsed();
            println!(
                "{} It took {:?} to process {} messages",
                self.name, duration, self.total_msgs
            );
        }
        if self.count == self.total_msgs {
            self.aggregator.send(Finish {})?;
        }
        Ok(ActorResult::Ok)
    }
}

#[derive(Actor, ActorFactory)]
struct Aggregator {
    ctx: ActorContext<Self>,
    total_actors: usize,
    name: String,
    #[tyra(default = "0")]
    actors_finished: usize,
    #[tyra(default = "Instant::now()")]
    start: Instant,
}

impl Handler<Finish> for Aggregator {
    fn handle(
        &mut self,
        _msg: Finish,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        self.actors_finished += 1;
        if self.actors_finished == self.total_actors {
            let duration = self.start.elapsed();
            println!(
                "{} It took {:?} to finish {} actors",
                self.name, duration, self.total_actors
            );
            self.ctx.system.stop(Duration::from_secs(60));
        }
        Ok(ActorResult::Ok)
    }
}

impl Handler<Start> for Aggregator {
    fn handle(
        &mut self,
        _msg: Start,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        sleep(Duration::from_secs((3) as u64));
        self.start = Instant::now();
        Ok(ActorResult::Ok)
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let message_count = 10000000;
    // ideal number is "amount of threads - 3"
    let actor_count = 7;

    let router_factory = RoundRobinRouter::factory();
    let router = actor_system
        .builder()
        .spawn("benchmark-router", router_factory)
        .unwrap();

    let aggregator = actor_system
        .builder()
        .spawn(
            "aggregator",
            Aggregator::factory(actor_count, String::from("aggregator")),
        )
        .unwrap();
    for i in 0..actor_count {
        let actor = actor_system
            .builder()
            .spawn(
                format!("benchmark-single-actor-{}", i),
                Benchmark::factory(
                    aggregator.clone(),
                    (message_count.clone() / actor_count.clone()) as usize,
                    format!("benchmark-{}", i),
                ),
            )
            .unwrap();

        router.send(AddActorMessage::new(actor)).unwrap();
    }

    println!("Actors have been created");
    let start = Instant::now();

    aggregator.send(Start {}).unwrap();
    let mut msgs = Vec::new();
    for _i in 0..message_count {
        let msg = MessageA {};
        msgs.push(msg);
    }
    router.send(BulkRouterMessage::new(msgs)).unwrap();

    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    exit(actor_system.await_shutdown());
}
