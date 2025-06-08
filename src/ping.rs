use std::time::Duration;
use crate::target::PingTarget;

#[derive(Debug)]
pub struct PingResults {
    pub target: PingTarget,
    pub responses: Vec<PingResponse>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub avg_duration: Option<Duration>,
    pub count_recv: u32,
    pub success_rate: f32,
    pub count_loss: u32,
    pub drop_rate: f32,
}

impl PingResults {
    pub fn new(target: PingTarget) -> Self {
        Self {
            target,
            responses: vec![],
            min_duration: None,
            max_duration: None,
            avg_duration: None,
            count_recv: 0,
            success_rate: 0.0,
            count_loss: 0,
            drop_rate: 0.0,
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

    fn update_rates(&mut self) {
        let total = self.count_recv + self.count_loss;
        if total == 0 {
            return;
        }
        self.success_rate = self.count_recv as f32 / total as f32;
        self.drop_rate = 1.0 - self.success_rate;
    }

    fn update_time_stats(&mut self, time: Duration) {
        self.min_duration = Some(self.min_duration.map_or(time, |min| min.min(time)));
        self.max_duration = Some(self.max_duration.map_or(time, |max| max.max(time)));
        self.avg_duration = Some(self.avg_duration.map_or(time, |avg| avg + time) / 2);
    }
}

#[derive(Debug)]
pub struct PingResponse {
    // index: u16,
    // size: usize,
    // ttl: u8,
    // sequence: PingSequence,
    pub duration: Duration,
}
