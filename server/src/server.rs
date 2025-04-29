use std::{
    net::{IpAddr, SocketAddr},
    path::Path, str::FromStr,
    sync::{atomic::{AtomicBool, Ordering}, Arc}
};
use crate::{
    handlers::{handle_block_request, handle_sync_request},
    helpers::{generate_dummy_crt, CERT_PATH, KEY_PATH}
};
use anyhow::{Context, Result};
use common::{
    message::{MessageHeader, MessageType, HEADER_LEN},
    helpers::unroll_anyhow_result,
};
use quinn::{
    crypto::rustls::QuicServerConfig,
    Endpoint, RecvStream, ServerConfig
};
use rustls_pki_types::{
    pem::PemObject,
    CertificateDer,
    PrivatePkcs8KeyDer
};

// TODO: inform client on internal server when possible

#[tokio::main]
pub async fn run(port: u16) -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");
    generate_dummy_crt().expect("");
    let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port);
    let server_config = get_server_config().expect("");
    let server = Endpoint::server(server_config, addr)
        .context("starting server")
        .expect("");

    println!("Running on port {port}");
    while let Some(conn) = server.accept().await {
        tokio::spawn(async move {
            match handle_connection(conn).await {
                Ok(_) => println!("Connection handled successfully"),
                Err(e) => println!("{}", unroll_anyhow_result(e)),
            }
        });
    }

    Ok(())
}

async fn handle_connection(connecting: quinn::Incoming) -> Result<()> {
    let connection = connecting.await?;

    let stop_signal = Arc::new(AtomicBool::new(false));

    let connection_for_monitoring = connection.clone();
    let stop_signal_clone = stop_signal.clone();
    tokio::spawn(async move {
        let stop_signal = stop_signal_clone;
        connection_for_monitoring.closed().await;
        stop_signal.store(true, Ordering::Relaxed);
    });

    println!("Connection established from: {}", connection.remote_address());

    while let Ok((send, recv)) = connection.accept_bi().await { tokio::spawn(handle_new_stream(send, recv, stop_signal.clone())); }

    Ok(())
}

async fn handle_new_stream(send: quinn::SendStream, mut recv: RecvStream, stop_signal: Arc<AtomicBool>) -> Result<()> {
    let mut header_buff = [0u8; HEADER_LEN];

    recv.read_exact(&mut header_buff)
        .await
        .context("reading header")?;
    let header = MessageHeader::deserialize(&header_buff)?;

    match header.msg_type {
        MessageType::SyncRequest => handle_sync_request(send, recv, &header, stop_signal.clone()).await,
        MessageType::BlockRequest => handle_block_request(send, recv, &header).await,
        _ => todo!("return error")
    }?;
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
