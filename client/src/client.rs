use std::sync::Arc;
use std::net::SocketAddr;
use crate::skip_cert::SkipServerVerification;
use anyhow::{Result, Context};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::Endpoint;

#[tokio::main]
pub async fn send_test_message() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");
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

    println!("Connected to server: {:?}", connection.remote_address());

    let (mut send, mut recv) = connection.open_bi().await?;

    send.write_all(b"Hello from QUIC client!").await.expect("writing");
    send.finish().context("closing tx")?;

    let buff = recv.read_to_end(1024).await?;
    println!("Received from server: {}", String::from_utf8_lossy(&buff));

    connection.close(0u32.into(), b"Done");

    println!("Connection closed");

    Ok(())
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
