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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_nanoseconds() {
        let d = Duration::ZERO;
        assert_eq!(d.display(), "0.00 ns");
    }

    #[test]
    fn display_microseconds() {
        let d = Duration::from_micros(500);
        assert_eq!(d.display(), "500.00 μs");
    }

    #[test]
    fn display_milliseconds() {
        let d = Duration::from_millis(500);
        assert_eq!(d.display(), "500.00 ms");
    }

    #[test]
    fn display_seconds() {
        let d = Duration::from_secs(2);
        assert_eq!(d.display(), "2.00 s");
    }

    #[test]
    fn display_one_second_boundary() {
        let d = Duration::from_millis(1000);
        assert_eq!(d.display(), "1.00 s");
    }

    #[test]
    fn display_sub_second() {
        let d = Duration::from_millis(150);
        assert_eq!(d.display(), "150.00 ms");
    }

    #[test]
    fn display_zero() {
        let d = Duration::ZERO;
        assert_eq!(d.display(), "0.00 ns");
    }

    #[test]
    fn display_one_microsecond_boundary() {
        let d = Duration::from_micros(1);
        assert_eq!(d.display(), "1.00 μs");
    }

    #[test]
    fn display_one_millisecond_boundary() {
        let d = Duration::from_millis(1);
        assert_eq!(d.display(), "1.00 ms");
    }

    #[test]
    fn display_just_under_millisecond() {
        let d = Duration::from_micros(999);
        assert_eq!(d.display(), "999.00 μs");
    }

    #[test]
    fn display_just_under_second() {
        let d = Duration::from_millis(999);
        assert_eq!(d.display(), "999.00 ms");
    }

    #[test]
    fn display_fractional_milliseconds() {
        let d = Duration::from_micros(1500);
        assert_eq!(d.display(), "1.50 ms");
    }

    #[test]
    fn display_fractional_seconds() {
        let d = Duration::from_micros(1_500_000);
        assert_eq!(d.display(), "1.50 s");
    }

    #[test]
    fn display_large_duration() {
        let d = Duration::from_secs(3600);
        assert_eq!(d.display(), "3600.00 s");
    }

    #[test]
    fn display_sub_microsecond_truncates_to_zero() {
        let d = Duration::from_nanos(500);
        assert_eq!(d.display(), "0.00 ns");
    }
}
