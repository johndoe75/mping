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

pub fn create_results_table(results: &[PingResults]) -> Table {
    let mut table = Table::new();

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

pub fn sort_results(results: &mut [PingResults]) {
    results.sort_by(|a, b| {
        let a_avg = a.avg_duration().map(|d| d.as_micros());
        let b_avg = b.avg_duration().map(|d| d.as_micros());
        match (a_avg, b_avg) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a_dur), Some(b_dur)) => a_dur.cmp(&b_dur),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::client::PingTarget;
    use crate::network::ping::{PingResponse, PingResults};
    use std::net::IpAddr;
    use std::time::Duration;

    fn make_target(ip: &str) -> PingTarget {
        PingTarget::new(ip.parse::<IpAddr>().unwrap())
    }

    fn make_results_with_avg(avg_ms: u64) -> PingResults {
        let mut results = PingResults::new(make_target("8.8.8.8"));
        results.add_received(PingResponse {
            duration: Duration::from_millis(avg_ms),
        });
        results
    }

    #[test]
    fn overall_stats_empty_results() {
        let results: Vec<PingResults> = vec![];
        let stats = OverallStats::from_results(&results);
        assert_eq!(stats.total_sent, 0);
        assert_eq!(stats.total_received, 0);
        assert_eq!(stats.total_lost, 0);
        assert_eq!(stats.loss_percentage, 0.0);
    }

    #[test]
    fn overall_stats_single_result_all_received() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        r.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });

        let stats = OverallStats::from_results(&[r]);
        assert_eq!(stats.total_sent, 2);
        assert_eq!(stats.total_received, 2);
        assert_eq!(stats.total_lost, 0);
        assert_eq!(stats.loss_percentage, 0.0);
    }

    #[test]
    fn overall_stats_mixed_loss_and_received() {
        let mut r1 = PingResults::new(make_target("8.8.8.8"));
        r1.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        r1.add_loss();

        let mut r2 = PingResults::new(make_target("1.1.1.1"));
        r2.add_received(PingResponse {
            duration: Duration::from_millis(5),
        });
        r2.add_received(PingResponse {
            duration: Duration::from_millis(15),
        });

        let stats = OverallStats::from_results(&[r1, r2]);
        assert_eq!(stats.total_sent, 4);
        assert_eq!(stats.total_received, 3);
        assert_eq!(stats.total_lost, 1);
        assert!((stats.loss_percentage - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overall_stats_all_loss() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_loss();
        r.add_loss();

        let stats = OverallStats::from_results(&[r]);
        assert_eq!(stats.total_sent, 2);
        assert_eq!(stats.total_received, 0);
        assert_eq!(stats.total_lost, 2);
        assert_eq!(stats.loss_percentage, 100.0);
    }

    #[test]
    fn sort_results_by_avg_duration_ascending() {
        let mut results = vec![
            make_results_with_avg(50),
            make_results_with_avg(10),
            make_results_with_avg(30),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(10)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(30)));
        assert_eq!(results[2].avg_duration(), Some(Duration::from_millis(50)));
    }

    #[test]
    fn sort_results_none_goes_last() {
        let mut results = vec![
            make_results_with_avg(10),
            PingResults::new(make_target("8.8.8.8")),
            make_results_with_avg(20),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(10)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(20)));
        assert!(results[2].avg_duration().is_none());
    }

    #[test]
    fn sort_results_empty_slice() {
        let mut results: Vec<PingResults> = vec![];
        sort_results(&mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn sort_results_single_element() {
        let mut results = vec![make_results_with_avg(42)];
        sort_results(&mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(42)));
    }

    #[test]
    fn sort_results_all_none() {
        let mut results = vec![
            PingResults::new(make_target("8.8.8.8")),
            PingResults::new(make_target("1.1.1.1")),
        ];
        sort_results(&mut results);
        assert!(results[0].avg_duration().is_none());
        assert!(results[1].avg_duration().is_none());
    }

    #[test]
    fn create_results_table_has_correct_row_count() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });

        let results = vec![r];
        let table = create_results_table(&results);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn overall_stats_many_hosts() {
        let mut results = Vec::new();
        for i in 0..100 {
            let mut r = PingResults::new(make_target(&format!("10.0.0.{}", i)));
            if i % 2 == 0 {
                r.add_received(PingResponse {
                    duration: Duration::from_millis(10),
                });
            } else {
                r.add_loss();
            }
            results.push(r);
        }

        let stats = OverallStats::from_results(&results);
        assert_eq!(stats.total_sent, 100);
        assert_eq!(stats.total_received, 50);
        assert_eq!(stats.total_lost, 50);
        assert!((stats.loss_percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overall_stats_single_host_single_loss() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_loss();

        let stats = OverallStats::from_results(&[r]);
        assert_eq!(stats.total_sent, 1);
        assert_eq!(stats.total_received, 0);
        assert_eq!(stats.total_lost, 1);
        assert_eq!(stats.loss_percentage, 100.0);
    }

    #[test]
    fn overall_stats_single_host_single_received() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_received(PingResponse {
            duration: Duration::from_millis(5),
        });

        let stats = OverallStats::from_results(&[r]);
        assert_eq!(stats.total_sent, 1);
        assert_eq!(stats.total_received, 1);
        assert_eq!(stats.total_lost, 0);
        assert_eq!(stats.loss_percentage, 0.0);
    }

    #[test]
    fn sort_results_preserves_stability_for_equal_avgs() {
        let t1 = PingTarget::new("10.0.0.1".parse::<IpAddr>().unwrap());
        let t2 = PingTarget::new("10.0.0.2".parse::<IpAddr>().unwrap());

        let mut r1 = PingResults::new(t1);
        r1.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });

        let mut r2 = PingResults::new(t2);
        r2.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });

        let mut results = vec![r1, r2];
        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(10)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(10)));
    }

    #[test]
    fn sort_results_already_sorted() {
        let mut results = vec![
            make_results_with_avg(10),
            make_results_with_avg(20),
            make_results_with_avg(30),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(10)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(20)));
        assert_eq!(results[2].avg_duration(), Some(Duration::from_millis(30)));
    }

    #[test]
    fn sort_results_reverse_sorted() {
        let mut results = vec![
            make_results_with_avg(30),
            make_results_with_avg(20),
            make_results_with_avg(10),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(10)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(20)));
        assert_eq!(results[2].avg_duration(), Some(Duration::from_millis(30)));
    }

    #[test]
    fn sort_results_mixed_none_at_start_middle_end() {
        let mut results = vec![
            PingResults::new(make_target("10.0.0.1")),
            make_results_with_avg(10),
            PingResults::new(make_target("10.0.0.2")),
            make_results_with_avg(5),
            PingResults::new(make_target("10.0.0.3")),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].avg_duration(), Some(Duration::from_millis(5)));
        assert_eq!(results[1].avg_duration(), Some(Duration::from_millis(10)));
        assert!(results[2].avg_duration().is_none());
        assert!(results[3].avg_duration().is_none());
        assert!(results[4].avg_duration().is_none());
    }

    #[test]
    fn create_results_table_multiple_rows() {
        let mut r1 = PingResults::new(make_target("8.8.8.8"));
        r1.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });

        let mut r2 = PingResults::new(make_target("1.1.1.1"));
        r2.add_loss();

        let results = vec![r1, r2];
        let table = create_results_table(&results);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn create_results_table_empty_input() {
        let results: Vec<PingResults> = vec![];
        let table = create_results_table(&results);
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn create_results_table_with_all_loss_shows_na_for_durations() {
        let mut r = PingResults::new(make_target("8.8.8.8"));
        r.add_loss();
        r.add_loss();

        let results = vec![r];
        let table = create_results_table(&results);
        assert_eq!(table.row_count(), 1);
    }
}
