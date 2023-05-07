use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::exit;
use std::time::Duration;
use tyra::prelude::*;

#[derive(Serialize, Deserialize, Hash, Clone)]
struct TestMsg {
    content: String,
    actor_wrapper: ActorWrapper<RemoteActor>,
}

impl ActorMessage for TestMsg {}

struct RemoteActor {}

impl Actor for RemoteActor {
    fn handle_serialized_message(
        &mut self,
        msg: SerializedMessage,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        let result = bincode::deserialize(&msg.content);
        if result.is_err() {
            return Ok(ActorResult::Ok);
        }
        let mut deserialized: TestMsg = result.unwrap();
        deserialized.actor_wrapper.init_after_deserialize(&context.system);
        deserialized.actor_wrapper.send_after(deserialized.clone(), Duration::from_millis(50))?;
        Ok(ActorResult::Ok)
    }
}

impl Handler<TestMsg> for RemoteActor {
    fn handle(
        &mut self,
        msg: TestMsg,
        context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        println!("{}", msg.content);
        context.system.stop(Duration::from_secs(10));
        Ok(ActorResult::Stop)
    }
}

struct RemoteActorFactory {}

impl ActorFactory<RemoteActor> for RemoteActorFactory {
    fn new_actor(
        &mut self,
        _context: ActorContext<RemoteActor>,
    ) -> Result<RemoteActor, Box<dyn Error>> {
        Ok(RemoteActor {})
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = RemoteActorFactory {};
    let remote_actor = actor_system.builder().spawn("hello-world", hw).unwrap();
    let msg = TestMsg {
        content: String::from("Hello World!"),
        actor_wrapper: remote_actor.clone(),
    };
    let serialized = bincode::serialize(&msg).unwrap();
    actor_system.send_to_address(remote_actor.get_address(), SerializedMessage::new(serialized));

    let result = actor_system.await_shutdown();

    exit(result);
}
