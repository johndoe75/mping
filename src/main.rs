use clap::Parser;
use colored::*;
use futures::future::join_all;
use rand::random;
use std::net::IpAddr;
use std::time::Duration;
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingIdentifier, PingSequence};
// use tokio::task::id;
use anyhow::{Result, anyhow};
use tokio::net::lookup_host;
use tokio::time;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(value_delimiter = ' ', num_args = 1..)]
    hosts: Option<Vec<String>>,

    #[clap(short, long)]
    cound: Option<u16>,

    #[clap(short, long)]
    delay: Option<f32>,
}

#[derive(Debug)]
struct PingTarget {
    host: Option<String>,
    addr: IpAddr,
}

impl PingTarget {
    fn display(&self) -> String {
        match &self.host {
            Some(host) => format!("{} ({})", host, self.addr),
            None => self.addr.to_string(),
        }
    }
}

#[derive(Debug)]
struct PingResults {
    target: PingTarget,
    responses: Vec<PingResponse>,
    count_received: u32,
    count_dropped: u32,
}

#[derive(Debug)]
struct PingResponse {
    index: u16,
    size: usize,
    ttl: u8,
    sequence: PingSequence,
    duration: Duration,
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let hosts = cli.hosts.unwrap_or_else(|| {
        eprintln!("{}", "Failed to parse hosts.");
        std::process::exit(1);
    });

    let number_pings = cli.cound.unwrap_or(5);
    let mut ping_delay = cli.delay.unwrap_or(1.0);
    if ping_delay < 0.25 {
        ping_delay = 0.25;
    }

    let mut targets = Vec::new();
    for host in hosts.iter() {
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

    let mut tasks = Vec::new();
    let client_v4 = Client::new(&Config::default()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let client_v6 = Client::new(&Config::builder().kind(ICMP::V6).build()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    for target in targets {
        let client = match target.addr {
            IpAddr::V4(_) => client_v4.clone(),
            IpAddr::V6(_) => client_v6.clone(),
        };

        tasks.push(tokio::spawn(ping(client, target, number_pings, ping_delay)));
    }

    let results = join_all(tasks).await;
    for result in results {
        let results: PingResults = result.unwrap();

        let success_rate = results.count_received as f32 / (results.count_received + results.count_dropped) as f32;
        let drop_rate = 1.0 - success_rate;

        let output_color = match drop_rate {
            drop_rate if drop_rate >= 0.1 && drop_rate < 0.75 => colored::Color::BrightYellow,
            drop_rate if drop_rate >= 0.75 => colored::Color::BrightRed,
            _ => colored::Color::BrightGreen,
        };

        let drop_rate_output =
            format!("{} % packets dropped", drop_rate * 100.0).color(output_color);

        println!(
            "{}: {} packets received, {} packets dropped, {}",
            results.target.display(),
            results.count_received,
            results.count_dropped,
            drop_rate_output
        );
    }
}

async fn ping(client: Client, target: PingTarget, count: u16, delay: f32) -> PingResults {
    let payload = [0; 56];
    let mut pinger = client.pinger(target.addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_millis((delay * 1000.0) as u64));

    let mut results = PingResults {
        target,
        responses: vec![],
        count_received: 0,
        count_dropped: 0,
    };

    for index in 0..count {
        interval.tick().await;
        match pinger.ping(PingSequence(index), &payload).await {
            Ok((IcmpPacket::V4(packet), duration)) => {
                let response = PingResponse {
                    index,
                    size: packet.get_size(),
                    ttl: packet.get_ttl().unwrap(),
                    sequence: packet.get_sequence(),
                    duration,
                };
                results.responses.push(response);
                results.count_received += 1;
            }
            Ok((IcmpPacket::V6(packet), duration)) => {
                let response = PingResponse {
                    index,
                    size: packet.get_size(),
                    ttl: 0,
                    sequence: packet.get_sequence(),
                    duration,
                };
                results.responses.push(response);
                results.count_received += 1;
            }
            Err(e) => {
                println!("{} ping error: {}", pinger.host, e);
                results.count_dropped += 1;
            }
        };
    }
    println!("[+] {} done.", pinger.host);
    results
}

async fn resolve_host(host: &str) -> anyhow::Result<PingTarget> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(PingTarget { host: None, addr: ip });
    }

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
