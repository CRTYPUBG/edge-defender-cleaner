// src/crypto.rs — AES decryption and MD5 verification

use anyhow::{Result, bail};
use aes::Aes256;
use cipher::{block_padding::Pkcs7, KeyIvInit, BlockModeDecrypt};
use md5::{Md5, Digest};

// Include the auto-generated crypto variables (AES_KEY, AES_IV, RES_MD5)
include!("crypto_key.rs");

type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// Verifies the MD5 hash of the payload, then decrypts it using AES-256-CBC.
pub fn decrypt_and_verify(encrypted_data: &[u8]) -> Result<Vec<u8>> {
    // 1. MD5 Verification
    let mut hasher = Md5::new();
    hasher.update(encrypted_data);
    let hash_result = hasher.finalize();
    let hash_hex = hex::encode(hash_result);

    if hash_hex.to_lowercase() != RES_MD5.to_lowercase() {
        bail!("Bütünlük kontrolü başarısız (MD5 uyuşmazlığı!). Dosya değiştirilmiş olabilir.");
    }

    // 2. AES-256-CBC Decryption
    let key = hex::decode(AES_KEY).map_err(|_| anyhow::anyhow!("Geçersiz AES Key hex"))?;
    let iv = hex::decode(AES_IV).map_err(|_| anyhow::anyhow!("Geçersiz AES IV hex"))?;

    let decryptor = Aes256CbcDec::new_from_slices(&key, &iv)
        .map_err(|_| anyhow::anyhow!("AES Init hatası: Geçersiz Key veya IV boyutu"))?;
    
    let decrypted = decryptor.decrypt_padded_vec::<Pkcs7>(encrypted_data)
        .map_err(|e| anyhow::anyhow!("Deşifreleme hatası: {:?}", e))?;

    Ok(decrypted)
}
