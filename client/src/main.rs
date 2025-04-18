mod skip_cert;
mod client;
mod command;
use client::send_test_message;

fn main() {
    send_test_message().expect("sending message");
}
