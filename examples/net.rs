use tyra::prelude::{ActorSystem, NetConfig, NetManagerFactory, NetProtocol, ThreadPoolConfig, TyraConfig};

fn main() {
    // generate config
    let mut actor_config = TyraConfig::new().unwrap();
    let cluster = ThreadPoolConfig::new(22, 4, 4, 1.00);
    actor_config.thread_pool.config.insert(String::from("mio"), cluster);
    // start system with config
    let actor_system = ActorSystem::new(actor_config);
    // create actor on the system
    let mut net_configs = Vec::new();
    net_configs.push(NetConfig::new(NetProtocol::TCP, "0.0.0.0", 2022));
    //net_configs.push(NetConfig::new(NetProtocol::TCP, "10.0.10.10", 2022));
    //net_configs.push(NetConfig::new(NetProtocol::UDP, "10.0.10.10", 2023));


    let _actor = actor_system
        .builder()
        .set_pool_name("mio")
        .spawn("test", NetManagerFactory::new(net_configs, 10))
        .unwrap();

    // send a message to the actor

    std::process::exit(actor_system.await_shutdown());
}

