use crate::error::Result;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex::FromHex;

/// Verify an ed25519 signature
/// used for validating discord webhooks
pub fn verify_sig(
    body: &str,
    signature: &str,
    timestamp: &str,
    public_key: String,
) -> Result<bool> {
    let public_key_bytes = <[u8; 32]>::from_hex(public_key)?;
    let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;

    // Decode the signature from hex
    let signature_bytes = <[u8; 64]>::from_hex(signature)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let timestamp_data = timestamp.as_bytes();
    let body_data = body.as_bytes();
    let message = [timestamp_data, body_data].concat();
    let res = public_key.verify(&message, &signature);
    Ok(res.is_ok())
}
