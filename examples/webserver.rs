use tyra::prelude::{ActorSystem, NetworkManagerFactory, ThreadPoolConfig, TyraConfig};

fn main() {
    // generate config
    let mut actor_config = TyraConfig::new().unwrap();
    let cluster = ThreadPoolConfig::new(18, 16, 16, 1.00);
    actor_config.thread_pool.config.insert(String::from("tcp-test"), cluster);
    // start system with config
    let actor_system = ActorSystem::new(actor_config);
    // create actor on the system
    let _actor = actor_system
        .builder()
        .set_pool_name("tcp-test")
        .spawn("test", NetworkManagerFactory::new(3))
        .unwrap();
    // send a message to the actor

    std::process::exit(actor_system.await_shutdown());
}

