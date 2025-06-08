use clap::Parser;
use futures::future::join_all;
use rand::random;
use std::net::IpAddr;
use std::time::Duration;
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingIdentifier, PingSequence};
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

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let hosts = cli.hosts.unwrap_or_else(|| {
        eprintln!("{}", "Failed to parse hosts.");
        std::process::exit(1);
    });

    let count = cli.cound.unwrap_or(5);
    let mut delay = cli.delay.unwrap_or(1.0);
    if delay < 0.25 {
        delay = 0.25;
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

    for host in hosts.iter() {
        match host.parse() {
            Ok(IpAddr::V4(addr)) => tasks.push(tokio::spawn(ping(
                client_v4.clone(),
                IpAddr::V4(addr),
                count,
                delay,
            ))),
            Ok(IpAddr::V6(addr)) => tasks.push(tokio::spawn(ping(
                client_v6.clone(),
                IpAddr::V6(addr),
                count,
                delay,
            ))),
            Err(e) => println!("{} parse to ipaddr error: {}", host, e),
        }
    }

    join_all(tasks).await;
}
async fn ping(client: Client, addr: IpAddr, count: u16, delay: f32) {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_millis((delay * 1000.0) as u64));
    for idx in 0..count {
        interval.tick().await;
        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(packet), dur)) => println!(
                "No.{}: {} bytes from {}: icmp_seq={} ttl={:?} time={:0.2?}",
                idx,
                packet.get_size(),
                packet.get_source(),
                packet.get_sequence(),
                packet.get_ttl(),
                dur
            ),
            Ok((IcmpPacket::V6(packet), dur)) => println!(
                "No.{}: {} bytes from {}: icmp_seq={} hlim={} time={:0.2?}",
                idx,
                packet.get_size(),
                packet.get_source(),
                packet.get_sequence(),
                packet.get_max_hop_limit(),
                dur
            ),
            Err(e) => println!("No.{}: {} ping {}", idx, pinger.host, e),
        };
    }
    println!("[+] {} done.", pinger.host);
}
