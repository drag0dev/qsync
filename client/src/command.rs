use clap::Parser;

#[derive(Parser)]
#[command(name = "fastsync")]
pub struct Command {
    #[arg(short, long)]
    pub remote_path: String,

    #[arg(short, long)]
    pub local_path: String,

    #[arg(short, long, default_value_t = 25)]
    pub concurrent_streams: usize,
}
