use std::net::IpAddr;

#[derive(Debug)]
pub struct PingTarget {
    pub host: Option<String>,
    pub addr: IpAddr,
}

impl PingTarget {
    pub fn display(&self) -> String {
        match &self.host {
            Some(host) => format!("{} ({})", host, self.addr),
            None => self.addr.to_string(),
        }
    }
}
