use std::path::Path;

use anyhow::{Result, Context};
use rcgen::{generate_simple_self_signed, CertifiedKey};

pub static CERT_PATH: &str = "./dummy_cert/temp_cert.pem";
pub static KEY_PATH: &str = "./dummy_cert/temp_key.pem";

pub fn generate_dummy_crt() -> Result<()> {
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(vec!["localhost".into()])
        .context("generating self signed cert")?;

    let cert_path = Path::new(CERT_PATH);
    let key_path = Path::new(KEY_PATH);

    std::fs::write(&cert_path, cert.pem())
        .context("writing cert")?;
    std::fs::write(&key_path, key_pair.serialize_pem())
        .context("writing keys")?;

    Ok(())
}

#[macro_export]
macro_rules! error_message {
    ( $tx: ident, $msg:expr ) => {{
        let msg = ErrorMessage::new($msg.into());
        let msg_ser = message_serialize_and_frame(MessageType::Error, &msg)?;
        $tx.write_all(&msg_ser).await.context("writing error msg")?;
    }};
}

