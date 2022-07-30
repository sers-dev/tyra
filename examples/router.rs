use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use tyra::prelude::{ActorFactory, ActorMessage, ActorSystem, ActorContext, Handler, TyraConfig, ActorMessageDeserializer};
use tyra::router::{
    AddActorMessage, RemoveActorMessage, RoundRobinRouterFactory, RouterMessage,
};

struct MessageA {}
impl ActorMessage for MessageA {}

struct HelloWorld {
    counter: usize,
}
impl ActorMessageDeserializer for HelloWorld {}

#[derive(Clone)]
struct HelloWorldFactory {}

impl ActorFactory<HelloWorld> for HelloWorldFactory {
    fn new_actor(&self, _context: ActorContext<HelloWorld>) -> HelloWorld {
        HelloWorld { counter: 0 }
    }
}
impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, _msg: MessageA, _context: &ActorContext<Self>) {
        self.counter += 1;
        println!("Received MSG {}", self.counter);
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = HelloWorldFactory {};
    let x = actor_system
        .builder()
        .set_mailbox_size(7)
        .spawn("hello-world-1", hw.clone()).unwrap();

    let y = actor_system
        .builder()
        .set_mailbox_size(7)
        .spawn("hello-world-2", hw).unwrap();

    let router_factory = RoundRobinRouterFactory::new();
    let router = actor_system.builder().spawn("hello-router", router_factory).unwrap();

    router.send(AddActorMessage::new(x));
    router.send(AddActorMessage::new(y.clone()));

    // the sleep is simply there, so that the output clearly shows the messages are routed round robin
    // since the routers work async it would not be visible otherwise
    router.send(RouterMessage::new(MessageA {}));
    router.send(RouterMessage::new(MessageA {}));
    sleep(Duration::from_millis(50));
    router.send(RouterMessage::new(MessageA {}));
    router.send(RouterMessage::new(MessageA {}));
    sleep(Duration::from_millis(50));
    router.send(RouterMessage::new(MessageA {}));
    router.send(RouterMessage::new(MessageA {}));

    router.send(RemoveActorMessage::new(y));
    router.send(RouterMessage::new(MessageA {}));
    router.send(RouterMessage::new(MessageA {}));

    actor_system.stop(Duration::from_secs(1));
    exit(actor_system.await_shutdown());
}
