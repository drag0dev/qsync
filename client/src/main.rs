use clap::Parser;
use std::path::Path;
use std::sync::Arc;
use std::net::SocketAddr;
use anyhow::{Result, Context};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{Connection, Endpoint};
use common::{
    helpers::unroll_anyhow_result,
    file::generate_temp_entry_point
};

mod skip_cert;
mod client;
mod command;
use client::{send_block_request, send_sync_request};
use command::Command;
use skip_cert::SkipServerVerification;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::parse();

    let local_path = Path::new(&cmd.local_path);
    if !local_path.exists() {
        println!("Error: local path is not valid");
        return Ok(());
    }

    let remote_path = Path::new(&cmd.remote_path);
    if !remote_path.is_absolute() {
        println!("Error: remote path has to be absolute");
        return Ok(());
    }

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

    let checksums = send_sync_request(&connection, &cmd.remote_path).await.context("sending sync request message");
    if let Err(e) = checksums {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }

    let checksums = checksums.unwrap();
    if checksums.is_none() {
        println!("Erorr: remote path does not exist ");
        return Ok(())
    }
    let checksums = checksums.unwrap();

    let temp_entry = generate_temp_entry_point(&cmd.local_path, &cmd.remote_path, &checksums.checksums)
        .context("generating temp entry point");
    if let Err(e) = temp_entry {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let temp_entry = temp_entry.unwrap();

    let first_file = checksums.checksums.first().unwrap();

    let block = send_block_request(&connection, &first_file.path, 0).await.context("getting a block");
    if let Err(e) = block {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let block = block.unwrap().unwrap();
    let data = block.block_data;

    let data = String::from_utf8_lossy(&data);
    println!("Received block: {data}");

    connection.close(0u32.into(), b"Done");

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
