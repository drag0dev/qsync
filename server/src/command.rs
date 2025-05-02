use clap::Parser;

#[derive(Parser)]
#[command(name = "qsync")]
pub struct Command {
    #[arg(short, long, default_value_t = 4433)]
    pub port: u16
}
