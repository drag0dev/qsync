use clap::Parser;

#[derive(Parser)]
#[command(name = "qsync", version = "1.0", about = "Sync your files moderately fast")]
pub struct Command {
    #[arg(short, long, help = "Server address")]
    pub server_address: String,

    #[arg(short, long, default_value_t = 4433, help = "Server port")]
    pub port: u16,

    #[arg(short, long, help = "Remote target")]
    pub remote_target: String,

    #[arg(short, long, help = "Local target, to be synced with the remote target")]
    pub local_target: String,

    #[arg(short, long, default_value_t = 25, help = "Number of QUIC streams to be used while syncing")]
    pub concurrent_streams: usize,

    #[arg(short, long, default_value_t = false, help = "Preserve timestamps from the remote target")]
    pub timestamp: bool,

    #[arg(long, default_value_t = false, help = "Preserve permissions from the remote target")]
    pub permissions: bool,

    #[arg(short, long, default_value_t = false, help = "Skip syncing any file or directory that has the same modified timestamp and size as the remote")]
    pub naive: bool,

    #[arg(short, long, default_value_t = 16384, help = "Size (in bytes) of each block used for comparing file differences")]
    pub block_size: usize,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
