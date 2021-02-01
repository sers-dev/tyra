use tractor::prelude::{TractorConfig, ActorSystem, ActorTrait};
use std::time::Duration;
use std::thread::sleep;

struct Message {
    text: String

}

impl Message {

}

struct HelloWorld {
    text: String
}

impl ActorTrait for HelloWorld {
}

fn main() {
    let actor_config = TractorConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.start();

    sleep(Duration::from_secs(3));

    actor_system.add_pool("aye", 7);
    actor_system.add_pool("aye", 100);


    actor_system.await_shutdown()

}