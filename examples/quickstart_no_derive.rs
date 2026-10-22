use std::error::Error;
use std::time::Duration;
use tyra::prelude::*;

struct TestMessage {}
impl TestMessage {
    pub fn new() -> Self {
        Self {}
    }
}
impl ActorMessage for TestMessage {}

struct TestActor {}
impl Actor for TestActor {}

struct TestActorFactory {}
impl ActorFactory<TestActor> for TestActorFactory {
    fn new_actor(
        &mut self,
        _context: ActorContext<TestActor>,
    ) -> Result<TestActor, Box<dyn Error>> {
        Ok(TestActor {})
    }
}

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
        .spawn("test", TestActorFactory {})
        .unwrap();
    // send a message to the actor
    actor.send(TestMessage::new()).unwrap();
    // wait for the system to stop
    std::process::exit(actor_system.await_shutdown());
}
