//! PQC module - DISABLED (MSVC build compatibility)
//! 
//! Post-quantum crypto (Dilithium2, Kyber1024) is disabled because the
//! pqcrypto crates include AVX2 assembly that MSVC cannot compile.
//! 
//! To re-enable:
//! 1. Install LLVM/Clang and set CC=clang
//! 2. Or use x86_64-pc-windows-gnu target with MinGW
//! 3. Or build on Linux and cross-compile

use anyhow::{Result, bail};
use std::path::Path;

pub async fn keygen_sig(_out_dir: &Path) -> Result<()> {
    bail!("PQC disabled: MSVC cannot compile pqcrypto AVX2 assembly. See pqc.rs for re-enable instructions.")
}

pub async fn sign_file(_sk_path: &Path, _in_path: &Path, _sig_out: &Path) -> Result<()> {
    bail!("PQC disabled: MSVC cannot compile pqcrypto AVX2 assembly.")
}

pub async fn verify_file(_pk_path: &Path, _in_path: &Path, _sig_path: &Path) -> Result<bool> {
    bail!("PQC disabled: MSVC cannot compile pqcrypto AVX2 assembly.")
}

pub async fn kem_demo(_out_dir: &Path) -> Result<()> {
    bail!("PQC disabled: MSVC cannot compile pqcrypto AVX2 assembly.")
}
