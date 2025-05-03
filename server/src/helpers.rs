use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use directories_next::ProjectDirs;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use tokio::fs;

pub async fn generate_dummy_crt() -> Result<(PathBuf, PathBuf)> {
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(vec!["localhost".into()])
        .context("generating self signed cert")?;

    let config_dir = ProjectDirs::from("", "", "qsync");
    if config_dir.is_none() { return Err(anyhow!("Creating config directory for qsync")); }
    let config_dir = config_dir.unwrap();
    let config_dir = config_dir.config_dir().to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .context("creating config dir for qsync")?;
    }

    let mut cert_path = config_dir.clone();
    cert_path.push("dummy_cert.pem");

    let mut key_path = config_dir;
    key_path.push("dummy_key.pem");

    if !cert_path.exists() || !key_path.exists() {
        fs::write(&cert_path, cert.pem())
            .await
            .context("writing cert")?;

        fs::write(&key_path, key_pair.serialize_pem())
            .await
            .context("writing keys")?;
    }
    Ok((cert_path, key_path))
}

#[macro_export]
macro_rules! error_message {
    ( $tx: ident, $msg:expr ) => {{
        let msg = ErrorMessage::new($msg.into());
        let msg_ser = message_serialize_and_frame(MessageType::Error, &msg, false).await?;
        $tx.write_all(&msg_ser).await.context("writing error msg")?;
    }};
}

