use clap::Parser;

#[derive(Parser)]
#[command(name = "fastsync")]
pub struct Command {
    #[arg(short, long)]
    pub server_address: String,

    #[arg(short, long, default_value_t = 4433)]
    pub port: u16,

    #[arg(short, long)]
    pub remote_path: String,

    #[arg(short, long)]
    pub local_path: String,

    #[arg(short, long, default_value_t = 25)]
    pub concurrent_streams: usize,

    #[arg(short, long, default_value_t = false)]
    pub timestamp: bool,
}
