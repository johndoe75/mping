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

impl std::fmt::Display for PingTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
