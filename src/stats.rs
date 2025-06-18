use crate::core::constants::PERCENTAGE_FACTOR;
use crate::display::DurationExt;
use crate::network::ping::PingResults;
use comfy_table::Table;

#[derive(Debug, Default)]
pub struct OverallStats {
    pub total_sent: u32,
    pub total_received: u32,
    pub total_lost: u32,
    pub loss_percentage: f64,
}

impl OverallStats {
    pub fn from_results(results: &[PingResults]) -> Self {
        let total_sent = results.iter().map(|r| r.total_count()).sum();
        let total_received = results.iter().map(|r| r.num_recv).sum();
        let total_lost = results.iter().map(|r| r.num_loss).sum();

        Self {
            total_sent,
            total_received,
            total_lost,
            loss_percentage: if total_sent > 0 {
                (total_lost as f64 / total_sent as f64) * PERCENTAGE_FACTOR
            } else {
                0.0
            },
        }
    }
}

pub fn create_results_table(results: &mut Vec<PingResults>) -> Table {
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
