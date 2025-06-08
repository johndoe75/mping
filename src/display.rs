use std::time::Duration;

pub trait DurationExt {
    fn display(&self) -> String;
}

impl DurationExt for Duration {
    fn display(&self) -> String {
        let millis = self.as_secs_f64() * 1000.0;

        match millis {
            m if m >= 1000.0 => format!("{:.2} s", m / 1000.0),
            m if m >= 1.0 => format!("{:.2} ms", m),
            m => format!("{:.2} μs", m * 1000.0),
        }
    }
}
