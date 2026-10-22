use std::error::Error;
use std::time::Duration;
use tyra::prelude::*;

// define an `ActorMessage` that can be sent to `Actors` that implement the corresponding `Handler<T>`
#[derive(ActorMessage)]
struct TestMessage {}
impl TestMessage {
    pub fn new() -> Self {
        Self {}
    }
}

// define the actual `Actor` that should process messages
#[derive(Actor, ActorFactory, Clone)]
struct TestActor {}
impl TestActor {
    pub fn new() -> Self {
        Self {}
    }
}

// implement our message for the `Actor`
impl Handler<TestMessage> for TestActor {
    fn handle(
        &mut self,
        _msg: TestMessage,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        println!("HELLO WORLD!");
        context.system.stop(Duration::from_millis(1000));
        Ok(ActorResult::Ok)
    }
}

fn main() {
    // generate config
    let actor_config = TyraConfig::new().unwrap();
    // start system with config
    let actor_system = ActorSystem::new(actor_config);
    // create actor on the system
    let actor = actor_system
        .builder()
        .spawn("test", TestActor::new())
        .unwrap();
    // send a message to the actor
    actor.send(TestMessage::new()).unwrap();
    // wait for the system to stop
    std::process::exit(actor_system.await_shutdown());
}
