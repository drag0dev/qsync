mod skip_cert;
mod client;
use client::send_test_message;

fn main() {
    send_test_message().expect("sending message");
}
