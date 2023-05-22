use std::time::Duration;
use log::warn;
use regex::Regex;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::Resolver;
use crate::config::tyra_config::{NET_CLUSTER_LB, NET_CLUSTER_POOL};
use crate::prelude::{ActorSystem, ClusterConfig, NetConfig, NetConnectionType, NetManagerFactory, NetProtocol, NetWorkerFactory};
use crate::router::{AddActorMessage, ShardedRouterFactory};

pub struct Cluster {

}

impl Cluster {

    fn generate_net_config(from: &Vec<String>, connection_type: NetConnectionType) -> Vec<NetConfig> {
        let regex = Regex::new("(tcp|udp):\\/\\/(.*):(.*)").unwrap();
        let mut net_configs = Vec::new();

        for host in from {
            let captures = regex.captures(host);
            if captures.is_none() {
                continue;
            }
            let captures = captures.unwrap();
            if captures.len() < 3 {
                continue;
            }
            let protocol = if &captures[1] == "tcp" {
                NetProtocol::TCP
            } else {
                NetProtocol::UDP
            };

            let port = if captures.len() == 4 {
                captures[3].parse::<usize>().unwrap()
            } else {
                2022 as usize
            };

            net_configs.push(NetConfig::new(protocol, connection_type, &captures[2], port));

        }
        return net_configs;
    }

    fn resolve_dns(from: &Vec<NetConfig>) -> Vec<NetConfig> {
        let mut to_return = Vec::new();
        for member in from {
            let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

            let response = resolver.lookup_ip(&member.host);
            match response {
                ResolveResult::Ok(addresses) => {
                    for address in addresses.iter() {
                        let mut res = member.clone();
                        res.host = format!("{}", address);
                        to_return.push(res);
                    }
                }
                _ => {
                    warn!("Can't find DNS records for '{}'", &member.host)
                }
            }
        }

        to_return.sort();
        to_return.dedup();
        return to_return;

    }

    pub fn init(system: &ActorSystem, cluster_config: &ClusterConfig) {

        let server_configs = Self::generate_net_config(&cluster_config.hosts, NetConnectionType::SERVER);
        let client_configs = Self::generate_net_config(&cluster_config.members, NetConnectionType::CLIENT);
        let client_configs = Self::resolve_dns(&client_configs);

        let worker_factory = NetWorkerFactory::new();
        let router_factory =  ShardedRouterFactory::new(false, false);
        let router = system.builder().set_pool_name(NET_CLUSTER_POOL).spawn(NET_CLUSTER_LB, router_factory).unwrap();

        let worker_count = system
            .get_available_actor_count_for_pool(NET_CLUSTER_POOL)
            .unwrap() - 1;
        let workers = system
            .builder()
            .set_pool_name(NET_CLUSTER_POOL)
            .spawn_multiple("cluster-worker", worker_factory.clone(), worker_count)
            .unwrap();
        for worker in &workers {
            router.send(AddActorMessage::new(worker.clone())).unwrap();
        }
        let _actor = system
            .builder()
            .set_pool_name(NET_CLUSTER_POOL)
            .spawn(
                "cluster-manager",
                NetManagerFactory::new(
                    server_configs,
                    Duration::from_secs(10),
                    Duration::from_secs(3),
                    workers,
                    router,
                ),
            )
            .unwrap();
    }
}