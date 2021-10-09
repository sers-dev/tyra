use std::process::exit;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tyra::prelude::{Actor, ActorFactory, ActorMessage, ActorSystem, ActorContext, Handler, TyraConfig, SerializedMessage};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct TestMsg {
    content: String
}

impl ActorMessage for TestMsg {}

#[derive(Clone)]
struct RemoteActor {
    ctx: ActorContext<Self>,
}

impl Actor for RemoteActor {
    fn handle_serialized_message(&self, msg: SerializedMessage) {
        let decoded :TestMsg = bincode::deserialize(&msg.content).unwrap();
        self.ctx.actor_ref.send(decoded)
    }
}

impl Handler<TestMsg> for RemoteActor {
    fn handle(&mut self, msg: TestMsg, _context: &ActorContext<Self>) {
        println!("{}", msg.content);
    }
}

struct RemoteActorFactory {}

impl ActorFactory<RemoteActor> for RemoteActorFactory {
    fn new_actor(&self, context: ActorContext<RemoteActor>) -> RemoteActor {
        RemoteActor { ctx: context }
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = RemoteActorFactory {};
    let x = actor_system
        .builder()
        .set_mailbox_size(7)
        .spawn("hello-world", hw).unwrap();
    let msg = TestMsg {
        content: String::from("Hello World!")
    };
    let serialized  = bincode::serialize(&msg).unwrap();
    actor_system.send_to_address(x.get_address(), SerializedMessage::new(serialized));

    actor_system.stop(Duration::from_secs(1));
    let result = actor_system.await_shutdown();

    exit(result);
}
