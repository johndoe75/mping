use crate::constants::PERCENTAGE_FACTOR;
use crate::ping::PingResults;

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
