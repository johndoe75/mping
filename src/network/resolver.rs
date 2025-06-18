use crate::core::config::PingConfig;
use crate::network::client::PingTarget;
use anyhow::anyhow;
use std::net::IpAddr;
use tokio::net::lookup_host;

pub async fn resolve_targets(ping_config: &PingConfig) -> Vec<PingTarget> {
    let mut targets = Vec::new();
    for host in ping_config.hosts.iter() {
        let target = match resolve_host(host).await {
            Ok(target) => target,
            Err(e) => {
                eprintln!("{} resolve error: {}", host, e);
                // Skip this host
                continue;
            }
        };
        targets.push(target);
    }
    targets
}

pub async fn resolve_host(host: &str) -> anyhow::Result<PingTarget> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(PingTarget {
            host: None,
            addr: ip,
        });
    }

    // Note: port 53 does not solve any purpose here, but it is required by tokio's lookup_host
    let mut addresses = lookup_host(format!("{}:53", host)).await?;

    // FIXME:
    //  For now, we take the first address from the results.  Later, we want to add the ability
    //  to specify which address type (v4, v6) to prefer, e.g. by using a command line flag. Also
    //  we might want to prefer IPv6 over IPv4 per default.
    addresses
        .next()
        .map(|addr| PingTarget {
            host: Some(host.to_string()),
            addr: addr.ip(),
        })
        .ok_or_else(|| anyhow!("{}: no address found", host))
}
