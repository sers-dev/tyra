use serde::{Deserialize, Serialize};
use std::process::exit;
use std::time::{Duration, Instant};
use tyra::prelude::{
    Actor, ActorContext, ActorFactory, ActorMessage, ActorSystem, Handler, SerializedMessage,
    TyraConfig,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct TestMsg {
    content: String,
}

impl ActorMessage for TestMsg {}

#[derive(Clone)]
struct RemoteActor {}

//impl Actor for RemoteActor {}

impl Actor for RemoteActor {
    fn handle_serialized_message(&mut self, msg: SerializedMessage, context: &ActorContext<Self>) {
        let decoded: TestMsg = bincode::deserialize(&msg.content).unwrap();
        context.actor_ref.send(decoded);
    }
    //fn pre_start(&mut self, _context: &ActorContext<Self>) {
    //    println!("WOOOT");
    //    _context.actor_ref.send(TestMsg{content: String::from("sers")})
    //}
    //fn post_stop(&mut self, _context: &ActorContext<Self>) where Self: Actor + Sized {
    //    println!("NICE");
    //}
}

impl Handler<TestMsg> for RemoteActor {
    fn handle(&mut self, msg: TestMsg, _context: &ActorContext<Self>) {
        println!("{}", msg.content);
    }
}

struct RemoteActorFactory {}

impl ActorFactory<RemoteActor> for RemoteActorFactory {
    fn new_actor(&self, _context: ActorContext<RemoteActor>) -> RemoteActor {
        RemoteActor {}
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = RemoteActorFactory {};
    let x = actor_system
        .builder()
        .set_mailbox_size(7)
        .spawn("hello-world", hw)
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
