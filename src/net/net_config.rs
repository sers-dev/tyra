#[derive(Clone)]
pub enum NetProtocol {
    UDP,
    TCP
}

#[derive(Clone)]
pub struct NetConfig {
    pub protocol: NetProtocol,
    pub host: String,
    pub port: usize,
}

impl NetConfig {
    pub fn new(protocol: NetProtocol, host: impl Into<String>, port: usize) -> Self {
        Self {
            protocol,
            host: host.into(),
            port,
        }
    }
}