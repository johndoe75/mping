use crate::core::constants::LOSS_TIMEOUT;
use crate::network::client::PingTarget;
use rand::random;
use std::time::Duration;
use surge_ping::{Client, IcmpPacket, PingIdentifier, PingSequence};
use tokio::time;

#[derive(Debug)]
pub struct PingResults {
    pub target: PingTarget,
    pub responses: Vec<PingResponse>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    avg_duration: Option<Duration>,
    pub num_recv: u32,
    recv_rate: f32,
    pub num_loss: u32,
    loss_rate: f32,
}

impl PingResults {
    pub fn new(target: PingTarget) -> Self {
        Self {
            target,
            responses: Vec::new(),
            min_duration: None,
            max_duration: None,
            avg_duration: None,
            num_recv: 0,
            recv_rate: 0.0,
            num_loss: 0,
            loss_rate: 0.0,
        }
    }

    pub fn recv_rate(&self) -> f32 {
        self.recv_rate
    }

    pub fn loss_rate(&self) -> f32 {
        self.loss_rate
    }

    pub fn avg_duration(&self) -> Option<Duration> {
        self.avg_duration
    }

    pub fn add_received(&mut self, response: PingResponse) {
        self.num_recv += 1;
        self.update_rates();
        self.update_time_stats(response.duration);
        self.responses.push(response);
    }

    pub fn add_loss(&mut self) {
        self.num_loss += 1;
        self.update_rates();
    }

    pub fn total_count(&self) -> u32 {
        self.num_recv + self.num_loss
    }

    fn update_rates(&mut self) {
        let total = self.total_count();
        if total == 0 {
            return;
        }
        self.recv_rate = self.num_recv as f32 / total as f32;
        self.loss_rate = 1.0 - self.recv_rate;
    }

    fn update_time_stats(&mut self, time: Duration) {
        self.min_duration = Some(self.min_duration.map_or(time, |min| min.min(time)));
        self.max_duration = Some(self.max_duration.map_or(time, |max| max.max(time)));

        if self.num_recv == 1 {
            self.avg_duration = Some(time);
        } else if let Some(current_avg) = self.avg_duration {
            let total_time = current_avg * (self.num_recv - 1) + time;
            self.avg_duration = Some(total_time / self.num_recv);
        }
    }
}

#[derive(Debug)]
pub struct PingResponse {
    pub duration: Duration,
}

pub async fn ping(client: Client, target: PingTarget, count: u16, delay: Duration) -> PingResults {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn make_target() -> PingTarget {
        PingTarget::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))
    }

    #[test]
    fn new_results_starts_empty() {
        let results = PingResults::new(make_target());
        assert_eq!(results.num_recv, 0);
        assert_eq!(results.num_loss, 0);
        assert_eq!(results.total_count(), 0);
        assert_eq!(results.recv_rate(), 0.0);
        assert_eq!(results.loss_rate(), 0.0);
        assert!(results.min_duration.is_none());
        assert!(results.max_duration.is_none());
        assert!(results.avg_duration().is_none());
    }

    #[test]
    fn add_received_updates_recv_count() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        assert_eq!(results.num_recv, 1);
        assert_eq!(results.num_loss, 0);
        assert_eq!(results.total_count(), 1);
    }

    #[test]
    fn add_loss_updates_loss_count() {
        let mut results = PingResults::new(make_target());
        results.add_loss();
        assert_eq!(results.num_recv, 0);
        assert_eq!(results.num_loss, 1);
        assert_eq!(results.total_count(), 1);
    }

    #[test]
    fn rates_are_correct_for_mixed_results() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_loss();
        results.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });

        assert_eq!(results.num_recv, 2);
        assert_eq!(results.num_loss, 1);
        assert_eq!(results.total_count(), 3);
        assert!((results.recv_rate() - 2.0 / 3.0).abs() < f32::EPSILON);
        assert!((results.loss_rate() - 1.0 / 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn time_stats_single_response() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(15),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(15)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(15)));
        assert_eq!(results.avg_duration(), Some(Duration::from_millis(15)));
    }

    #[test]
    fn time_stats_multiple_responses() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(30),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(10)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(30)));
        assert_eq!(results.avg_duration(), Some(Duration::from_millis(20)));
    }

    #[test]
    fn time_stats_updates_min_and_max() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(50),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(100),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(10)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(100)));
    }

    #[test]
    fn rates_are_zero_when_no_packets() {
        let results = PingResults::new(make_target());
        assert_eq!(results.recv_rate(), 0.0);
        assert_eq!(results.loss_rate(), 0.0);
    }

    #[test]
    fn all_loss_results_in_100_percent_loss() {
        let mut results = PingResults::new(make_target());
        results.add_loss();
        results.add_loss();

        assert_eq!(results.num_recv, 0);
        assert_eq!(results.num_loss, 2);
        assert_eq!(results.loss_rate(), 1.0);
        assert_eq!(results.recv_rate(), 0.0);
    }

    #[test]
    fn all_received_results_in_zero_percent_loss() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });

        assert_eq!(results.loss_rate(), 0.0);
        assert_eq!(results.recv_rate(), 1.0);
    }

    #[test]
    fn time_stats_with_zero_duration() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::ZERO,
        });

        assert_eq!(results.min_duration, Some(Duration::ZERO));
        assert_eq!(results.max_duration, Some(Duration::ZERO));
        assert_eq!(results.avg_duration(), Some(Duration::ZERO));
    }

    #[test]
    fn time_stats_with_sub_millisecond_durations() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_micros(100),
        });
        results.add_received(PingResponse {
            duration: Duration::from_micros(200),
        });

        assert_eq!(results.min_duration, Some(Duration::from_micros(100)));
        assert_eq!(results.max_duration, Some(Duration::from_micros(200)));
        assert_eq!(results.avg_duration(), Some(Duration::from_micros(150)));
    }

    #[test]
    fn time_stats_with_large_durations() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_secs(5),
        });
        results.add_received(PingResponse {
            duration: Duration::from_secs(10),
        });

        assert_eq!(results.min_duration, Some(Duration::from_secs(5)));
        assert_eq!(results.max_duration, Some(Duration::from_secs(10)));
        let avg = results.avg_duration().unwrap();
        assert_eq!(avg.as_secs(), 7);
        assert_eq!(avg.subsec_millis(), 500);
    }

    #[test]
    fn time_stats_with_identical_durations() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(42),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(42),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(42),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(42)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(42)));
        assert_eq!(results.avg_duration(), Some(Duration::from_millis(42)));
    }

    #[test]
    fn time_stats_with_decreasing_durations() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(100),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(50),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(10)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(100)));
        let avg = results.avg_duration().unwrap();
        assert_eq!(avg.as_millis(), 53);
    }

    #[test]
    fn time_stats_interleaved_loss_does_not_affect_durations() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_loss();
        results.add_loss();
        results.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });

        assert_eq!(results.min_duration, Some(Duration::from_millis(10)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(20)));
        assert_eq!(results.avg_duration(), Some(Duration::from_millis(15)));
        assert_eq!(results.num_recv, 2);
        assert_eq!(results.num_loss, 2);
    }

    #[test]
    fn responses_vector_tracks_all_received() {
        let mut results = PingResults::new(make_target());
        results.add_received(PingResponse {
            duration: Duration::from_millis(10),
        });
        results.add_received(PingResponse {
            duration: Duration::from_millis(20),
        });

        assert_eq!(results.responses.len(), 2);
        assert_eq!(results.responses[0].duration, Duration::from_millis(10));
        assert_eq!(results.responses[1].duration, Duration::from_millis(20));
    }

    #[test]
    fn loss_does_not_add_to_responses() {
        let mut results = PingResults::new(make_target());
        results.add_loss();
        results.add_loss();

        assert!(results.responses.is_empty());
    }

    #[test]
    fn avg_calculation_does_not_overflow_with_many_packets() {
        let mut results = PingResults::new(make_target());
        for _ in 0..1000 {
            results.add_received(PingResponse {
                duration: Duration::from_millis(1),
            });
        }

        assert_eq!(results.num_recv, 1000);
        assert_eq!(results.avg_duration(), Some(Duration::from_millis(1)));
        assert_eq!(results.min_duration, Some(Duration::from_millis(1)));
        assert_eq!(results.max_duration, Some(Duration::from_millis(1)));
    }
}
