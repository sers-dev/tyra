use std::process::exit;
use std::time::{Duration, Instant};
use tyra::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, ActorResult, ActorSystem, ActorWrapper, Handler, TyraConfig};
use tyra::router::{AddActorMessage, RoundRobinRouterFactory, RouterMessage};

struct MessageA {}

impl ActorMessage for MessageA {}

struct Finish {}

impl ActorMessage for Finish {}

struct Start {}

impl ActorMessage for Start {}

struct Benchmark {
    aggregator: ActorWrapper<Aggregator>,
    total_msgs: usize,
    name: String,
    count: usize,
    start: Instant,
}

struct BenchmarkFactory {
    total_msgs: usize,
    aggregator: ActorWrapper<Aggregator>,
    name: String,
}

impl ActorFactory<Benchmark> for BenchmarkFactory {
    fn new_actor(&self, _context: ActorContext<Benchmark>) -> Benchmark {
        Benchmark::new(self.total_msgs, self.name.clone(), self.aggregator.clone())
    }
}

impl Benchmark {
    pub fn new(total_msgs: usize, name: String, aggregator: ActorWrapper<Aggregator>) -> Self {
        Self {
            aggregator,
            total_msgs,
            name,
            count: 0,
            start: Instant::now(),
        }
    }
}

impl Actor for Benchmark {}

impl Handler<MessageA> for Benchmark {
    fn handle(&mut self, _msg: MessageA, _context: &ActorContext<Self>) -> ActorResult {
        if self.count == 0 {
            //sleep(Duration::from_secs((3) as u64));
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
            self.aggregator.send(Finish {}).unwrap();
        }
        ActorResult::Ok
    }
}

struct Aggregator {
    ctx: ActorContext<Self>,
    total_actors: usize,
    name: String,
    actors_finished: usize,
    start: Instant,
}

struct AggregatorFactory {
    total_actors: usize,
    name: String,
}

impl Aggregator {
    pub fn new(total_actors: usize, name: String, context: ActorContext<Self>) -> Self {
        Self {
            ctx: context,
            total_actors,
            name,
            actors_finished: 0,
            start: Instant::now(),
        }
    }
}

impl Actor for Aggregator {}

impl ActorFactory<Aggregator> for AggregatorFactory {
    fn new_actor(&self, context: ActorContext<Aggregator>) -> Aggregator {
        Aggregator::new(self.total_actors, self.name.clone(), context)
    }
}

impl Handler<Finish> for Aggregator {
    fn handle(&mut self, _msg: Finish, _context: &ActorContext<Self>) -> ActorResult {
        self.actors_finished += 1;
        if self.actors_finished == self.total_actors {
            let duration = self.start.elapsed();
            println!(
                "{} It took {:?} to finish {} actors",
                self.name, duration, self.total_actors
            );
            self.ctx.system.stop(Duration::from_secs(60));
        }
        ActorResult::Ok
    }
}

impl Handler<Start> for Aggregator {
    fn handle(&mut self, _msg: Start, _context: &ActorContext<Self>) -> ActorResult {
        //sleep(Duration::from_secs((3) as u64));
        self.start = Instant::now();
        ActorResult::Ok
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let message_count = 10000000;
    // ideal number is "amount of threads - 3"
    let actor_count = 10;

    let router_factory = RoundRobinRouterFactory::new();
    let router = actor_system
        .builder()
        .spawn("benchmark-router", router_factory)
        .unwrap();

    let aggregator = actor_system
        .builder()
        .spawn(
            "aggregator",
            AggregatorFactory {
                total_actors: actor_count,
                name: String::from("aggregator"),
            },
        )
        .unwrap();
    for i in 0..actor_count {
        let actor = actor_system
            .builder()
            .spawn(
                format!("benchmark-single-actor-{}", i),
                BenchmarkFactory {
                    name: String::from(format!("benchmark-{}", i)),
                    total_msgs: (message_count.clone() / actor_count.clone()) as usize,
                    aggregator: aggregator.clone(),
                },
            )
            .unwrap();
        router.send(AddActorMessage::new(actor)).unwrap();
    }

    println!("Actors have been created");
    let start = Instant::now();

    aggregator.send(Start {}).unwrap();
    for _i in 0..message_count {
        let msg = MessageA {};
        router.send(RouterMessage::new(msg)).unwrap();
    }
    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    exit(actor_system.await_shutdown());
}
