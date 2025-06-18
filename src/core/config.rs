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

    #[clap(short, long)]
    pub delay: Option<f32>,
}

#[derive(Debug)]
pub struct PingConfig {
    pub hosts: Vec<String>,
    pub packet_count: u16,
    pub interval: Duration,
}

impl PingConfig {
    pub fn from_args(args: Args) -> Result<Self> {
        let hosts = args.hosts.ok_or_else(|| anyhow!("No hosts specified."))?;
        let count = args.count.unwrap_or(5);
        let delay = Duration::from_secs_f32(args.delay.unwrap_or(1.0).max(0.1));

        Ok(Self {
            hosts,
            packet_count: count,
            interval: delay,
        })
    }
}
