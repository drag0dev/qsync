use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf, str::FromStr,
    sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration
};
use crate::{
    handlers::{handle_block_request, handle_sync_request},
    helpers::generate_dummy_crt
};
use anyhow::{Context, Result};
use common::{
    message::{MessageHeader, MessageType, HEADER_LEN},
    helpers::unroll_anyhow_result,
};
use quinn::{
    crypto::rustls::QuicServerConfig, Endpoint, IdleTimeout,
    RecvStream, ServerConfig, TransportConfig
};
use rustls_pki_types::{
    pem::PemObject,
    CertificateDer,
    PrivatePkcs8KeyDer
};

#[tokio::main]
pub async fn run(port: u16) -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");
    let (cert_path, key_path) = generate_dummy_crt()
        .await
        .context("creating dummy certs")?;

    let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port);
    let server_config = get_server_config(cert_path, key_path).expect("");
    let server = Endpoint::server(server_config, addr)
        .context("starting server")?;

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

    let res = recv.read_exact(&mut header_buff)
        .await
        .context("reading header");
    if let Err(e) = res {
        println!("Internal error: {}", unroll_anyhow_result(e));
        return Ok(());
    }

    let header = MessageHeader::deserialize(&header_buff);
    if let Err(e) = header {
        println!("Internal error: {}", unroll_anyhow_result(e));
        return Ok(());
    }
    let header = header.unwrap();

    let e = match header.msg_type {
        MessageType::SyncRequest => handle_sync_request(send, recv, &header, stop_signal.clone()).await,
        MessageType::BlockRequest => handle_block_request(send, recv, &header).await,
        _ => todo!("return error"),
    };
    if let Err(e) = e {
        println!("Internal error: {}", unroll_anyhow_result(e));
    }
    Ok(())
}

fn get_server_config(cert_path: PathBuf, key_path: PathBuf) -> Result<quinn::ServerConfig> {
    let cert_chain = CertificateDer::from_pem_file(cert_path)
        .context("decoding cert")?;

    let keys = PrivatePkcs8KeyDer::from_pem_file(key_path)
        .context("decoding private key")?;

    let crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_chain], keys.into())
        .context("creating rustls server config")?;

    let crypto = QuicServerConfig::try_from(crypto)
        .context("converting config into quic config")?;

    let mut server_config = ServerConfig::with_crypto(Arc::new(crypto));

    let mut transport_config = TransportConfig::default();
    let timeout: IdleTimeout = Duration::from_secs(120)
        .try_into()
        .context("creating idle timeout")?;
    transport_config.max_idle_timeout(Some(timeout));

    server_config.transport_config(Arc::new(transport_config));

    Ok(server_config)
}
