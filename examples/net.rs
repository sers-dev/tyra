use simple_logger::SimpleLogger;
use tyra::prelude::*;

fn main() {
    SimpleLogger::new().init().unwrap();

    // generate config
    let mut actor_config = TyraConfig::new().unwrap();
    actor_config.cluster.enabled = true;
    let cluster = ThreadPoolConfig::new(22, 4, 4, 1.00);
    actor_config
        .thread_pool
        .config
        .insert(String::from("mio"), cluster);
    // start system with config
    let actor_system = ActorSystem::new(actor_config);


    std::process::exit(actor_system.await_shutdown());
}
