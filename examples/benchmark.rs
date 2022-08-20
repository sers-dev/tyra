use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tyra::prelude::{Actor, ActorContext, ActorFactory, ActorMessage, ActorResult, ActorSystem, Handler, TyraConfig};

struct MessageA {}

impl ActorMessage for MessageA {}

struct Benchmark {
    total_msgs: usize,
    name: String,
    count: usize,
    start: Instant,
}

struct BenchmarkFactory {
    total_msgs: usize,
    name: String,
}

impl ActorFactory<Benchmark> for BenchmarkFactory {
    fn new_actor(&self, context: ActorContext<Benchmark>) -> Benchmark {
        Benchmark::new(self.total_msgs, self.name.clone(), context)
    }
}

impl Benchmark {
    pub fn new(total_msgs: usize, name: String, _context: ActorContext<Self>) -> Self {
        Self {
            total_msgs,
            name,
            count: 0,
            start: Instant::now(),
        }
    }
}

impl Actor for Benchmark {}

impl Handler<MessageA> for Benchmark {
    fn handle(&mut self, _msg: MessageA, context: &ActorContext<Self>) -> ActorResult {
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
        ActorResult::Ok
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let message_count = 10000000;

    let actor = actor_system
        .builder()
        .spawn(
            "benchmark-single-actor",
            BenchmarkFactory {
                name: String::from("benchmark"),
                total_msgs: message_count as usize,
            },
        )
        .unwrap();
    println!("Actors have been created");
    let start = Instant::now();

    for _i in 0..message_count {
        let msg = MessageA {};
        actor.send(msg);
    }
    let duration = start.elapsed();
    println!("It took {:?} to send {} messages", duration, message_count);

    exit(actor_system.await_shutdown());
}
