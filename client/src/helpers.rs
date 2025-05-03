use crate::skip_cert::SkipServerVerification;
use quinn::crypto::rustls::QuicClientConfig;
use tokio::fs::File;
use std::time::{Duration, SystemTime};
use std::{path::PathBuf, sync::Arc};
use std::net::{SocketAddr, ToSocketAddrs};
use quinn::{Connection, Endpoint, IdleTimeout, TransportConfig};
use anyhow::{anyhow, Context, Result};

pub async fn get_connection(server_address: &str, port: u16) -> Result<Connection> {
    let client_config = configure_client()
        .context("configuring client")?;

    let mut client = Endpoint::client("0.0.0.0:0".parse()?)?;
    client.set_default_client_config(client_config);

    let server_address = format!("{server_address}:{port}");
    let server_address = &server_address.to_socket_addrs()
        .context("parsing server address")?
        .into_iter()
        .find(|addr| { if let SocketAddr::V4(_) = addr { true } else { false } });

    if server_address.is_none() {
        return Err(anyhow!("cannot resolve provided server address"));
    }
    let server_address = server_address.unwrap();

    let connection = client.connect(server_address, "localhost")
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

    let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));

    let mut transport_config = TransportConfig::default();
    let timeout: IdleTimeout = Duration::from_secs(120)
        .try_into()
        .context("creating idle timeout")?;
    transport_config.max_idle_timeout(Some(timeout));

    client_config.transport_config(Arc::new(transport_config));

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

pub async fn clean_up_temp(temp_entry: &PathBuf) -> Result<()> {
    if temp_entry.is_dir() {
        tokio::fs::remove_dir_all(temp_entry).await
    } else {
        tokio::fs::remove_file(temp_entry).await
    }.context("deleting temp entry")?;
    Ok(())
}
