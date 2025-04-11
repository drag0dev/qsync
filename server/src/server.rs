use std::{
    error::Error,
    path::Path,
    sync::Arc
};
use crate::helpers::{generate_dummy_crt, CERT_PATH, KEY_PATH};
use anyhow::{Context, Result};
use quinn::{
    crypto::rustls::QuicServerConfig,
    Endpoint,
    ServerConfig
};
use rustls_pki_types::{
    pem::PemObject,
    CertificateDer,
    PrivatePkcs8KeyDer
};


#[tokio::main]
pub async fn run() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");
    generate_dummy_crt().expect("");
    let server_config = get_server_config().expect("");
    let server = Endpoint::server(server_config, "127.0.0.1:4433".parse().unwrap())
        .context("starting server")
        .expect("");

    println!("Running on port 4433");
    while let Some(conn) = server.accept().await {
        tokio::spawn(async move {
            match handle_connection(conn).await {
                Ok(_) => println!("Connection handled successfully"),
                Err(e) => eprintln!("Connection error: {}", e),
            }
        });
    }

    Ok(())
}

async fn handle_connection(connecting: quinn::Incoming) -> Result<(), Box<dyn Error>> {
    let connection = connecting.await?;
    println!("Connection established from: {}", connection.remote_address());

    while let Ok((mut send, mut recv)) = connection.accept_bi().await {
        let buff = recv.read_to_end(1024).await.expect("reading response");
        println!("Received from client: {}", String::from_utf8_lossy(&buff));
        send.write_all(b"Hello from QUIC server!").await?;
        send.finish().context("closing tx")?;
    }

    Ok(())
}

fn get_server_config() -> Result<quinn::ServerConfig> {
    let cert_chain = CertificateDer::from_pem_file(Path::new(CERT_PATH))
        .context("decoding cert")?;

    let keys = PrivatePkcs8KeyDer::from_pem_file(Path::new(KEY_PATH))
        .context("decoding private key")?;

    let crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_chain], keys.into())
        .context("creating rustls server config")?;

    let crypto = QuicServerConfig::try_from(crypto)
        .context("converting config into quic config")?;

    let server_config = ServerConfig::with_crypto(Arc::new(crypto));

    Ok(server_config)
}
