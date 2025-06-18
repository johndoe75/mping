use crate::core::config::PingConfig;
use crate::network::client::PingTarget;
use anyhow::anyhow;
use std::net::IpAddr;
use tokio::net::lookup_host;

pub async fn resolve_targets(ping_config: &PingConfig) -> Vec<PingTarget> {
    let mut targets = Vec::new();

    for host in ping_config.hosts.iter() {
        let target: PingTarget;
        if let Some(ping_target) = try_parse_ip_target(host) {
            target = reverse_resolve_ip(ping_target.addr).await.unwrap();
        } else {
            target = match resolve_hostname(host).await {
                Ok(target) => target,
                Err(e) => {
                    eprintln!("{} resolve error: {}", host, e);
                    // Skip this host
                    continue;
                }
            };
        }
        targets.push(target);
    }
    targets
}

/// Returns `Some(PingTarget)` if `host` is an IP address, `None` otherwise.
fn try_parse_ip_target(host: &str) -> Option<PingTarget> {
    // If this is an IP address, we can skip the DNS lookup
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(PingTarget {
            host: None,
            addr: ip,
        });
    }
    None
}

pub async fn resolve_hostname(hostname: &str) -> anyhow::Result<PingTarget> {
    // Note: port 53 does not solve any purpose here, but it is required by tokio's lookup_host
    let mut addresses = lookup_host(format!("{}:53", hostname)).await?;

    // FIXME:
    //  For now, we take the first address from the results.  Later, we want to add the ability
    //  to specify which address type (v4, v6) to prefer, e.g. by using a command line flag. Also
    //  we might want to prefer IPv6 over IPv4 per default.
    addresses
        .next()
        .map(|addr| PingTarget {
            host: Some(hostname.to_string()),
            addr: addr.ip(),
        })
        .ok_or_else(|| anyhow!("{}: no address found", hostname))
}

pub async fn reverse_resolve_ip(addr: IpAddr) -> anyhow::Result<PingTarget> {
    let ip_addr = addr.to_string();
    // FIXME: lookup_host does only IP -> hostname, not the other way around.
    let mut host_names = lookup_host(format!("{}:53", ip_addr)).await?;

    host_names
        .next()
        .map(|hostname| PingTarget {
            host: Some(hostname.to_string()),
            addr: addr,
        })
        .ok_or_else(|| anyhow!("{}: no hostname found", ip_addr))
}
