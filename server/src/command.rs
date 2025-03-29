use clap::Parser;

#[derive(Parser)]
#[command(name = "fastsync")]
pub struct Command {
    #[arg(short, long, default_value_t = 443)]
    port: u16
}
