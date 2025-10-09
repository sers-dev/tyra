#[derive(Clone, Ord, Eq, PartialOrd, PartialEq, Copy, Debug)]
pub enum NetProtocol {
    TCP,
    UDP,
}

#[derive(Clone, Ord, Eq, PartialOrd, PartialEq, Copy, Debug)]
pub enum NetConnectionType {
    CLIENT,
    SERVER
}

#[derive(Clone, Ord, Eq, PartialOrd, PartialEq, Debug)]
pub struct NetConfig {
    pub protocol: NetProtocol,
    pub connection_type: NetConnectionType,
    pub host: String,
    pub port: usize,
}

impl NetConfig {
    pub fn new(protocol: NetProtocol, connection_type: NetConnectionType, host: impl Into<String>, port: usize) -> Self {
        Self {
            protocol,
            connection_type,
            host: host.into(),
            port,
        }
    }
}
