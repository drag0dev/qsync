use clap::Parser;
use anyhow::Context;

mod helpers;
mod command;
mod server;
mod handlers;

use command::Command;
use server::run;
use common::helpers::unroll_anyhow_result;

fn main() {
    let cmd = Command::parse();
    let res = run(cmd.port).context("running quic endpoint");
    if let Err(e) = res {
        println!("{}", unroll_anyhow_result(e));
    }
}
