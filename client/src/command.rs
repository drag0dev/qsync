use clap::Parser;

#[derive(Parser)]
#[command(name = "fastsync")]
pub struct Command {
    #[arg(short, long)]
    pub remote_path: String,
    #[arg(short, long)]
    pub local_path: String
}
