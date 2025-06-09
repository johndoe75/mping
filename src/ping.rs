use crate::target::PingTarget;
use std::time::Duration;

#[derive(Debug)]
pub struct PingResults {
    pub target: PingTarget,
    pub responses: Vec<PingResponse>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub avg_duration: Option<Duration>,
    pub count_recv: u32,
    pub recv_rate: f32,
    pub count_loss: u32,
    pub loss_rate: f32,
}

impl PingResults {
    pub fn new(target: PingTarget) -> Self {
        Self {
            target,
            responses: Vec::new(),
            min_duration: None,
            max_duration: None,
            avg_duration: None,
            count_recv: 0,
            recv_rate: 0.0,
            count_loss: 0,
            loss_rate: 0.0,
        }
    }

    pub fn add_success(&mut self, response: PingResponse) {
        self.count_recv += 1;
        self.update_rates();
        self.update_time_stats(response.duration);
        self.responses.push(response);
    }

    pub fn add_drop(&mut self) {
        self.count_loss += 1;
        self.update_rates();
    }

    pub fn total_count(&self) -> u32 {
        self.count_recv + self.count_loss
    }

    fn update_rates(&mut self) {
        let total = self.total_count();
        if total == 0 {
            return;
        }
        self.recv_rate = self.count_recv as f32 / total as f32;
        self.loss_rate = 1.0 - self.recv_rate;
    }

    fn update_time_stats(&mut self, time: Duration) {
        self.min_duration = Some(self.min_duration.map_or(time, |min| min.min(time)));
        self.max_duration = Some(self.max_duration.map_or(time, |max| max.max(time)));

        if self.count_recv == 1 {
            self.avg_duration = Some(time);
        } else if let Some(current_avg) = self.avg_duration {
            let total_time = current_avg * (self.count_recv - 1) + time;
            self.avg_duration = Some(total_time / self.count_recv);
        }
    }
}

#[derive(Debug)]
pub struct PingResponse {
    pub duration: Duration,
}
