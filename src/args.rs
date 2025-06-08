use clap::Parser;

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
