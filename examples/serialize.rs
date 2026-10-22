use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::exit;
use std::time::{Duration, Instant};
use tyra::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, ActorMessage)]
struct TestMsg {
    content: String,
}

#[derive(Clone, Actor, ActorFactory)]
struct RemoteActor {}

impl Handler<TestMsg> for RemoteActor {
    fn handle(
        &mut self,
        msg: TestMsg,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        println!("{}", msg.content);
        Ok(ActorResult::Ok)
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let x = actor_system
        .builder()
        .spawn("hello-world", RemoteActor::factory())
        .unwrap();
    let msg = TestMsg {
        content: String::from("Hello World!"),
    };
    let serialized = bincode::serialize(&msg).unwrap();
    actor_system.send_to_address(x.get_address(), SerializedMessage::new(serialized));
    let start = Instant::now();

    actor_system.stop(Duration::from_secs(10));
    let result = actor_system.await_shutdown();

    let duration = start.elapsed();
    println!("It took {:?} to send stop", duration);

    exit(result);
}
