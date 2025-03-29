mod command;
use clap::Parser;
use command::Command;

fn main() {
    let cmd = Command::parse();
}
