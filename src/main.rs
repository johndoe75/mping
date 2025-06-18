use clap::Parser;
use comfy_table::ContentArrangement;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use futures::future::join_all;
use mping::core::config::Args;
use mping::core::config::PingConfig;
use mping::display::DurationExt;
use mping::network::client::PingClients;
use mping::network::ping;
use mping::network::resolver::resolve_targets;
use mping::stats;
use mping::stats::OverallStats;

type Result<T> = anyhow::Result<T>;

#[tokio::main]
async fn main() -> Result<()> {
    let config = PingConfig::from_args(Args::parse())?;
    let clients = PingClients::new()?;
    let targets = resolve_targets(&config).await;

    println!(
        "PING {} hosts with {} packets each in {} intervals ...",
        targets.len(),
        config.packet_count,
        config.interval.display()
    );

    let tasks = targets
        .into_iter()
        .map(|target| {
            let client = clients.get_client(target.addr).clone();
            tokio::spawn(ping::ping(
                client,
                target,
                config.packet_count,
                config.interval,
            ))
        })
        .collect::<Vec<_>>();

    let mut results = join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    let overall_stats = OverallStats::from_results(&results);

    let mut table = stats::create_results_table(&mut results);
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
