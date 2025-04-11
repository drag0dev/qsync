use clap::Parser;

mod helpers;
mod command;
mod server;

use command::Command;
use server::run;

fn main() {
    let cmd = Command::parse();
    run().expect("running quic endpoint");
}
