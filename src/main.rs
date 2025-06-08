use clap::Parser;
use colored::*;
use futures::future::join_all;
use rand::random;
use std::net::IpAddr;
use std::time::Duration;
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingIdentifier, PingSequence};
// use tokio::task::id;
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
struct PingResponse {
    index: u16,
    size: usize,
    ttl: u8,
    sequence: PingSequence,
    duration: Duration,
}

#[derive(Debug)]
struct PingResults {
    addr: IpAddr,
    responses: Vec<PingResponse>,
    count_ok: u32,
    count_nok: u32,
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

    let results = join_all(tasks).await;
    for result in results {
        let results: PingResults = result.unwrap();

        let success_rate = results.count_ok as f32 / (results.count_ok + results.count_nok) as f32;
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
            results.addr, results.count_ok, results.count_nok, drop_rate_output
        );
    }
}

async fn ping(client: Client, addr: IpAddr, count: u16, delay: f32) -> PingResults {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_millis((delay * 1000.0) as u64));

    let mut results = PingResults {
        addr,
        responses: vec![],
        count_ok: 0,
        count_nok: 0,
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
                results.count_ok += 1;
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
                results.count_ok += 1;
            }
            Err(e) => {
                println!("{} ping error: {}", pinger.host, e);
                results.count_nok += 1;
            }
        };
    }
    println!("[+] {} done.", pinger.host);
    results
}
