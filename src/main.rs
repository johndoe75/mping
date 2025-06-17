use anyhow::anyhow;
use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{ContentArrangement, Table};
use futures::future::join_all;
use mping::args::Args;
use mping::constants::{LOSS_TIMEOUT, PERCENTAGE_FACTOR};
use mping::display::DurationExt;
use mping::ping::{PingResponse, PingResults};
use mping::stats::OverallStats;
use mping::target::PingTarget;
use rand::random;
use std::net::IpAddr;
use std::time::Duration;
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingIdentifier, PingSequence};
use tokio::net::lookup_host;
use tokio::time;

type Result<T> = anyhow::Result<T>;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();

    let hosts = cli.hosts.ok_or_else(|| anyhow!("No hosts specified."))?;

    let number_pings = cli.count.unwrap_or(5);
    let mut ping_delay = cli.delay.unwrap_or(1.0);
    if ping_delay < 0.1 {
        ping_delay = 0.1;
    }
    let ping_delay = Duration::from_secs_f32(ping_delay);

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

    println!(
        "PING {} hosts with {} packets each in {} intervals ...",
        targets.len(),
        number_pings,
        ping_delay.display()
    );

    let client_v4 = Client::new(&Config::default()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let client_v6 = Client::new(&Config::builder().kind(ICMP::V6).build()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let mut tasks = Vec::new();
    for target in targets {
        let client = match target.addr {
            IpAddr::V4(_) => client_v4.clone(),
            IpAddr::V6(_) => client_v6.clone(),
        };

        tasks.push(tokio::spawn(ping(client, target, number_pings, ping_delay)));
    }

    let mut results = join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    let overall_stats = OverallStats::from_results(&results);

    let mut table = create_results_table(&mut results);
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Host", "Addr", "Sent", "Recv", "Loss", "Min", "Max", "Avg",
        ]);

    print!("\n{}\n\n", table);
    println!(
        "Overall {} sent, {} received ({:.2} % loss)",
        overall_stats.total_sent, overall_stats.total_received, overall_stats.loss_percentage
    );

    Ok(())
}

async fn ping(client: Client, target: PingTarget, count: u16, delay: Duration) -> PingResults {
    let payload = [0; 56];
    let mut pinger = client.pinger(target.addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(LOSS_TIMEOUT as u64));
    let mut interval = time::interval(delay);

    let mut results: PingResults = PingResults::new(target);

    for index in 0..count {
        interval.tick().await;
        match pinger.ping(PingSequence(index), &payload).await {
            Ok((IcmpPacket::V4(_), duration)) => {
                let response = PingResponse { duration };

                results.add_received(response);
            }
            Ok((IcmpPacket::V6(_), duration)) => {
                let response = PingResponse { duration };

                results.add_received(response);
            }
            Err(e) => {
                println!("{} ping error: {}", pinger.host, e);
                results.add_loss();
            }
        };
    }
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

fn create_results_table(results: &mut Vec<PingResults>) -> Table {
    let mut table = Table::new();

    let results = sort_results(results);

    for result in results {
        table.add_row(vec![
            result.target.host.as_deref().unwrap_or("-"),
            &result.target.addr.to_string(),
            &result.total_count().to_string(),
            &result.num_recv.to_string(),
            &format!("{:.1}%", result.loss_rate() * PERCENTAGE_FACTOR as f32),
            &result
                .min_duration
                .map(|d| d.display())
                .unwrap_or_else(|| "N/A".to_string()),
            &result
                .max_duration
                .map(|d| d.display())
                .unwrap_or_else(|| "N/A".to_string()),
            &result
                .avg_duration()
                .map(|d| d.display())
                .unwrap_or_else(|| "N/A".to_string()),
        ]);
    }

    table
}

fn sort_results(results: &mut Vec<PingResults>) -> &Vec<PingResults> {
    results.sort_by(|a, b| {
        let a_avg = a.avg_duration().map(|d| d.as_micros());
        let b_avg = b.avg_duration().map(|d| d.as_micros());
        // Vergleiche die Durchschnittswerte, behandle None als größter Wert
        match (a_avg, b_avg) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a_dur), Some(b_dur)) => a_dur.cmp(&b_dur),
        }
    });
    results
}
