use anyhow::{Result, anyhow};
use std::net::IpAddr;
use surge_ping::{Client, Config, ICMP};

pub struct PingClients {
    v4: Client,
    v6: Client,
}

impl PingClients {
    pub fn new() -> Result<Self> {
        let v4 = Client::new(&Config::default())
            .map_err(|e| anyhow!("Failed to create IPv4 client: {}", e))?;
        let v6 = Client::new(&Config::builder().kind(ICMP::V6).build())
            .map_err(|e| anyhow!("Failed to create IPv6 client: {}", e))?;

        Ok(Self { v4, v6 })
    }

    pub fn get_client(&self, addr: IpAddr) -> &Client {
        match addr {
            IpAddr::V4(_) => &self.v4,
            IpAddr::V6(_) => &self.v6,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn ping_target_new_has_no_host() {
        let addr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let target = PingTarget::new(addr);
        assert_eq!(target.host, None);
        assert_eq!(target.addr, addr);
    }

    #[test]
    fn ping_target_with_host_sets_host() {
        let addr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let target = PingTarget::with_host("cloudflare.com".to_string(), addr);
        assert_eq!(target.host, Some("cloudflare.com".to_string()));
        assert_eq!(target.addr, addr);
    }

    #[test]
    fn ping_target_display_without_host() {
        let addr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let target = PingTarget::new(addr);
        assert_eq!(format!("{}", target), "8.8.8.8");
    }

    #[test]
    fn ping_target_display_with_host() {
        let addr = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let target = PingTarget::with_host("localhost".to_string(), addr);
        assert_eq!(format!("{}", target), "localhost (::1)");
    }
}
