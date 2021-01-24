use tractor::prelude::{TractorConfig, ActorSystem};

fn main() {
    let actor_config = TractorConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    actor_system.start();
    actor_system.await_shutdown()

}