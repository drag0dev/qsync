use clap::Parser;
use std::sync::Arc;
use std::net::SocketAddr;
use anyhow::{Result, Context};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{Connection, Endpoint};
use common::helpers::unroll_anyhow_result;

mod skip_cert;
mod client;
mod command;
use client::send_sync_request;
use command::Command;
use skip_cert::SkipServerVerification;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::parse();

    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");

    let connection = get_connection()
        .await
        .context("connecting to the server");
    if let Err(e) = connection {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let connection = connection.unwrap();

    println!("Connected to server: {:?}", connection.remote_address());

    let res = send_sync_request(&connection, &cmd.path).await.context("sending sync request message");
    if let Err(e) = res { println!("{}", unroll_anyhow_result(e)); }
    Ok(())
}

async fn get_connection() -> Result<Connection> {
    let client_config = configure_client()
        .context("configuring client")?;

    let mut client = Endpoint::client("0.0.0.0:0".parse()?)?;
    client.set_default_client_config(client_config);

    let server_addr = "127.0.0.1:4433".parse::<SocketAddr>()
        .context("parsing server address")?;

    let connection = client.connect(server_addr, "localhost")
        .context("establishing connecting to server")?
        .await
        .context("connecting to server")?;

    Ok(connection)
}

fn configure_client() -> Result<quinn::ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let crypto = QuicClientConfig::try_from(crypto)
        .context("creating quic config")?;

    let client_config = quinn::ClientConfig::new(Arc::new(crypto));

    Ok(client_config)
}
