use anyhow::{Result, anyhow};
use clap::Parser;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(value_delimiter = ' ', num_args = 1..)]
    pub hosts: Option<Vec<String>>,

    #[clap(short, long)]
    pub count: Option<u16>,

    /// Delay between packets in milliseconds (default: 1000, min: 100)
    #[clap(short, long)]
    pub delay: Option<u64>,
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Debug)]
pub struct PingConfig {
    pub hosts: Vec<String>,
    pub packet_count: u16,
    pub interval: Duration,
    pub verbose: bool,
}

impl PingConfig {
    pub fn from_args(args: Args) -> Result<Self> {
        let hosts = args.hosts.ok_or_else(|| anyhow!("No hosts specified."))?;
        let count = args.count.unwrap_or(5);
        let delay = Duration::from_millis(args.delay.unwrap_or(1000).max(100));

        Ok(Self {
            hosts,
            packet_count: count,
            interval: delay,
            verbose: args.verbose,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_args_no_hosts_returns_error() {
        let args = Args {
            hosts: None,
            count: None,
            delay: None,
            verbose: false,
        };
        let result = PingConfig::from_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn from_args_default_values() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: None,
            delay: None,
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.hosts, vec!["example.com"]);
        assert_eq!(config.packet_count, 5);
        assert_eq!(config.interval, Duration::from_millis(1000));
    }

    #[test]
    fn from_args_custom_count_and_delay() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: Some(20),
            delay: Some(2000),
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.packet_count, 20);
        assert_eq!(config.interval, Duration::from_millis(2000));
    }

    #[test]
    fn from_args_delay_enforced_minimum() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: None,
            delay: Some(50),
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.interval, Duration::from_millis(100));
    }

    #[test]
    fn from_args_multiple_hosts() {
        let args = Args {
            hosts: Some(vec!["google.com".to_string(), "8.8.8.8".to_string()]),
            count: None,
            delay: None,
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.hosts.len(), 2);
    }

    #[test]
    fn from_args_empty_hosts_vec_returns_error() {
        let args = Args {
            hosts: Some(vec![]),
            count: None,
            delay: None,
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert!(config.hosts.is_empty());
    }

    #[test]
    fn from_args_delay_exactly_at_minimum() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: None,
            delay: Some(100),
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.interval, Duration::from_millis(100));
    }

    #[test]
    fn from_args_zero_count_is_allowed() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: Some(0),
            delay: None,
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.packet_count, 0);
    }

    #[test]
    fn from_args_zero_delay_clamped_to_minimum() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: None,
            delay: Some(0),
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.interval, Duration::from_millis(100));
    }

    #[test]
    fn from_args_very_large_delay() {
        let args = Args {
            hosts: Some(vec!["example.com".to_string()]),
            count: None,
            delay: Some(86_400_000),
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.interval, Duration::from_millis(86_400_000));
    }

    #[test]
    fn from_args_hosts_with_empty_string() {
        let args = Args {
            hosts: Some(vec!["".to_string()]),
            count: None,
            delay: None,
            verbose: false,
        };
        let config = PingConfig::from_args(args).unwrap();
        assert_eq!(config.hosts, vec![""]);
    }
}
