use std::time::Duration;

pub trait DurationExt {
    fn display(&self) -> String;
}

impl DurationExt for Duration {
    fn display(&self) -> String {
        let micros = self.as_micros() as f64;

        match micros {
            m if m >= 1_000_000.0 => format!("{:.2} s", m / 1_000_000.0),
            m if m >= 1_000.0 => format!("{:.2} ms", m / 1_000.0),
            m if m >= 1.0 => format!("{:.2} μs", m),
            m => format!("{:.2} ns", m * 1000.0),
        }
    }
}
