use clap::Parser;

#[derive(Parser)]
#[command(name = "fastsync")]
pub struct Command {
    #[arg(short, long)]
    pub path: String
}
