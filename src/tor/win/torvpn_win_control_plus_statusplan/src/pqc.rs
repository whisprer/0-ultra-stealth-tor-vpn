use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use pqcrypto_dilithium::dilithium2;
use pqcrypto_dilithium::dilithium2::{PublicKey, SecretKey, SignedMessage};
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{Ciphertext, SharedSecret};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, SignedMessage as SignSignedMessage};
use std::path::Path;
use tokio::fs;

pub async fn keygen_sig(out_dir: &Path) -> Result<()> {
    let (pk, sk) = dilithium2::keypair();
    tokio::fs::create_dir_all(out_dir).await?;
    tokio::fs::write(out_dir.join("pqc_dilithium2.pk"), pk.as_bytes()).await?;
    tokio::fs::write(out_dir.join("pqc_dilithium2.sk"), sk.as_bytes()).await?;
    Ok(())
}
pub async fn sign_file(sk_path: &Path, in_path: &Path, sig_out: &Path) -> Result<()> {
    let sk_bytes = fs::read(sk_path).await
        .with_context(|| format!("reading dilithium SK at {}", sk_path.display()))?;
    let sk = SecretKey::from_bytes(&sk_bytes)
        .context("parsing dilithium SK bytes")?;
    let data = fs::read(in_path).await?;
    let sm: SignedMessage = dilithium2::sign(&data, &sk);
    fs::write(sig_out, sm.as_bytes()).await?;
    Ok(())
}
pub async fn verify_file(pk_path: &Path, in_path: &Path, sig_path: &Path) -> Result<bool> {
    let pk_bytes = fs::read(pk_path).await
        .with_context(|| format!("reading dilithium PK at {}", pk_path.display()))?;
    let pk = PublicKey::from_bytes(&pk_bytes)
        .context("parsing dilithium PK bytes")?;
    let data = fs::read(in_path).await?;
    let sig = fs::read(sig_path).await
        .with_context(|| format!("reading signature at {}", sig_path.display()))?;

    // sign_file() writes a SignedMessage (not a detached signature).
    // Verify by opening the signed message and comparing to the expected plaintext.
    let sm = SignedMessage::from_bytes(&sig)
        .context("parsing SignedMessage bytes")?;
    match dilithium2::open(&sm, &pk) {
        Ok(opened) => Ok(opened.as_slice() == data.as_slice()),
        Err(_) => Ok(false),
    }
}
pub async fn kem_demo(out_dir: &Path) -> Result<()> {
    let (pk, sk) = kyber1024::keypair();
    // pqcrypto-kyber 0.8 returns (SharedSecret, Ciphertext)
    let (ss_enc, ct) = kyber1024::encapsulate(&pk);
    let ss_dec = kyber1024::decapsulate(&ct, &sk);
    tokio::fs::create_dir_all(out_dir).await?;
    tokio::fs::write(out_dir.join("kyber_ct.bin"), ct.as_bytes()).await?;
    tokio::fs::write(out_dir.join("kyber_ss.enc"), general_purpose::STANDARD.encode(ss_enc.as_bytes())).await?;
    tokio::fs::write(out_dir.join("kyber_ss.dec"), general_purpose::STANDARD.encode(ss_dec.as_bytes())).await?;
    Ok(())
}
