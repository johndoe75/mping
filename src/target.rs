use std::net::IpAddr;

#[derive(Debug)]
pub struct PingTarget {
    pub host: Option<String>,
    pub addr: IpAddr,
}

impl PingTarget {
    pub fn new(addr: IpAddr) -> Self {
        Self { host: None, addr }
    }

    pub fn with_host(host: String, addr: IpAddr) -> Self {
        Self {
            host: Some(host),
            addr,
        }
    }
}

impl std::fmt::Display for PingTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.host {
            Some(host) => write!(f, "{} ({})", host, self.addr),
            None => write!(f, "{}", self.addr),
        }
    }
}
