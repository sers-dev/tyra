use tyractorsaur::prelude::{MessageTrait, ActorTrait, ActorProps, Context, Handler, TyractorsaurConfig, ActorSystem, RoundRobinRouterProps, RouterMessage, AddActorMessage, RemoveActorMessage};
use std::time::Duration;
use std::process::exit;
use std::thread::sleep;

struct MessageA {}
impl MessageTrait for MessageA {}

struct HelloWorld {
    counter: usize,
}
impl ActorTrait for HelloWorld {}

#[derive(Clone)]
struct HelloWorldProps {}

impl ActorProps<HelloWorld> for HelloWorldProps {
    fn new_actor(&self, _context: Context<HelloWorld>) -> HelloWorld {
        HelloWorld {
            counter: 0,
        }
    }
}
impl Handler<MessageA> for HelloWorld {
    fn handle(&mut self, _msg: MessageA, _context: &Context<Self>) {
        self.counter += 1;
        println!("Received MSG {}", self.counter);
    }
}


fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = HelloWorldProps {};
    let x = actor_system
        .builder("hello-world-1")
        .set_mailbox_size(7)
        .build(hw.clone());

    let y = actor_system
        .builder("hello-world-2")
        .set_mailbox_size(7)
        .build(hw);

    let router_props = RoundRobinRouterProps::new();
    let router  = actor_system
        .builder("hello-router")
        .build(router_props);

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
