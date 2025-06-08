use clap::Parser;
use colored::*;
use futures::future::join_all;
use rand::random;
use std::net::IpAddr;
use std::time::Duration;
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingIdentifier, PingSequence};
// use tokio::task::id;
use anyhow::anyhow;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
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
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    avg_duration: Option<Duration>,
    count_recv: u32,
    success_rate: f32,
    count_loss: u32,
    drop_rate: f32,
}

impl PingResults {
    fn new(target: PingTarget) -> Self {
        Self {
            target,
            responses: vec![],
            min_duration: None,
            max_duration: None,
            avg_duration: None,
            count_recv: 0,
            success_rate: 0.0,
            count_loss: 0,
            drop_rate: 0.0,
        }
    }

    fn add_success(&mut self, response: PingResponse) {
        self.count_recv += 1;
        self.update_rates();
        self.update_time_stats(response.duration);
        self.responses.push(response);
    }

    fn add_drop(&mut self) {
        self.count_loss += 1;
        self.update_rates();
    }

    fn update_rates(&mut self) {
        let total = self.count_recv + self.count_loss;
        if total == 0 {
            return;
        }
        self.success_rate = self.count_recv as f32 / total as f32;
        self.drop_rate = 1.0 - self.success_rate;
    }

    fn update_time_stats(&mut self, time: Duration) {
        self.min_duration = Some(self.min_duration.map_or(time, |min| min.min(time)));
        self.max_duration = Some(self.max_duration.map_or(time, |max| max.max(time)));
        self.avg_duration = Some(self.avg_duration.map_or(time, |avg| avg + time) / 2);
    }
}

// Extend the duration type with a human-readable output of a duration.
trait DurationExt {
    fn human_readable(&self) -> String;
}

impl DurationExt for Duration {
    fn human_readable(&self) -> String {
        let millis = self.as_secs_f64() * 1000.0;

        match millis {
            m if m >= 1000.0 => format!("{:.2} s", m / 1000.0),
            m if m >= 1.0 => format!("{:.2} ms", m),
            m => format!("{:.2} μs", m * 1000.0),
        }
    }
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

    let mut table = Table::new();
    // table.load_preset("compact");
    table.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Host", "IP", "Sent", "Recv", "Loss", "Avg"]);

    for host_ping_result in results {
        let results: PingResults = host_ping_result.unwrap();

        table.add_row(vec![
            results.target.host.unwrap_or_else(|| "-".to_string()),
            results.target.addr.to_string(),
            (results.count_recv + results.count_loss).to_string(),
            results.count_recv.to_string(),
            results.count_loss.to_string(),
            results.avg_duration.map(|d| d.human_readable()).unwrap_or_else(|| "N/A".to_string()),
        ]);
    }
    println!("{}", table);
}

async fn ping(client: Client, target: PingTarget, count: u16, delay: f32) -> PingResults {
    let payload = [0; 56];
    let mut pinger = client.pinger(target.addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_millis((delay * 1000.0) as u64));

    let mut results: PingResults = PingResults::new(target);

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

                results.add_success(response);
            }
            Ok((IcmpPacket::V6(packet), duration)) => {
                let response = PingResponse {
                    index,
                    size: packet.get_size(),
                    ttl: 0,
                    sequence: packet.get_sequence(),
                    duration,
                };

                results.add_success(response);
            }
            Err(e) => {
                println!("{} ping error: {}", pinger.host, e);
                results.add_drop();
            }
        };
    }
    println!("[+] {} done.", pinger.host);
    results
}

async fn resolve_host(host: &str) -> anyhow::Result<PingTarget> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(PingTarget {
            host: None,
            addr: ip,
        });
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
