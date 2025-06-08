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
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let hosts = cli.hosts.unwrap_or_else(|| {
        eprintln!("{}", "Failed to parse hosts.");
        std::process::exit(1);
    });

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
            Ok(IpAddr::V4(addr)) => {
                tasks.push(tokio::spawn(ping(client_v4.clone(), IpAddr::V4(addr))))
            }
            Ok(IpAddr::V6(addr)) => {
                tasks.push(tokio::spawn(ping(client_v6.clone(), IpAddr::V6(addr))))
            }
            Err(e) => println!("{} parse to ipaddr error: {}", host, e),
        }
    }

    join_all(tasks).await;
}
async fn ping(client: Client, addr: IpAddr) {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_secs(1));
    for idx in 0..5 {
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
