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
