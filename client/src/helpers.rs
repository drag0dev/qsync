use crate::skip_cert::SkipServerVerification;
use quinn::crypto::rustls::QuicClientConfig;
use tokio::fs::File;
use std::time::SystemTime;
use std::{path::PathBuf, sync::Arc};
use std::net::SocketAddr;
use quinn::{Connection, Endpoint};
use anyhow::{Result, Context};

pub async fn get_connection(server_address: &str, port: u16) -> Result<Connection> {
    let client_config = configure_client()
        .context("configuring client")?;

    let mut client = Endpoint::client("0.0.0.0:0".parse()?)?;
    client.set_default_client_config(client_config);

    let server_address = format!("{server_address}:{port}");
    let server_addr = server_address.parse::<SocketAddr>()
        .context("parsing server address")?;

    let connection = client.connect(server_addr, "localhost")
        .context("establishing connecting to server")?
        .await
        .context("connecting to server")?;

    Ok(connection)
}

pub fn configure_client() -> Result<quinn::ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let crypto = QuicClientConfig::try_from(crypto)
        .context("creating quic config")?;

    let client_config = quinn::ClientConfig::new(Arc::new(crypto));

    Ok(client_config)
}

/// checks if the local file has matching size ajnd modified timestamp
pub async fn naive_check(path: &PathBuf, remote_size: u64, remote_modified_timestamp: u128) -> Result<bool> {
    let file = File::open(path)
        .await
        .context("opening local file for naive check")?;

    let meta = file
        .metadata()
        .await
        .context("getting metadata")?;

    if meta.len() != remote_size { return Ok(false); }

    let local_modified_timestamp = meta
        .modified()
        .context("getting local modified time")?
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("getting local modified timestamp")?
        .as_millis();

    if local_modified_timestamp != remote_modified_timestamp { return Ok(false); }

    Ok(true)
}
