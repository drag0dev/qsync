use crate::skip_cert::SkipServerVerification;
use quinn::crypto::rustls::QuicClientConfig;
use std::sync::Arc;
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
