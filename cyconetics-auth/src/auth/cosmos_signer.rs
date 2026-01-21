use anyhow::{Result, Context};
use bip39::{Mnemonic, Language};
use cosmrs::bip32::{self, Prefix};
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::tendermint::PrivateKey;
use keyring::Entry;
use sha2::{Sha256, Digest};
use std::str::FromStr;
use zeroize::Zeroize;

/// Service name for keyring storage (unique per deployment)
const KEYRING_SERVICE: &str = "cyconetics.auth.bostrom";
/// Entry label for the mnemonic
const KEYRING_ENTRY: &str = "primary_seed";

/// Authentication payload structure (extend with ALN/qpudatashard fields as needed)
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthPayload {
    pub session_id: String,
    pub device_id: String,
    pub timestamp: u64,
    pub xr_zone: String,  // XR-Grid zone tag for jurisdiction binding
    pub nonce: [u8; 32],
}

/// Generate new mnemonic and store in OS keyring (run once per identity)
pub fn generate_and_store_identity() -> Result<(Mnemonic, String)> {
    let mnemonic = Mnemonic::generate_in(Language::English, 24)?;
    let mnemonic_phrase = mnemonic.phrase().to_string();

    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ENTRY)?;
    entry.set_password(&mnemonic_phrase)?;

    // Derive address for confirmation (using bostrom prefix if applicable, fallback "cosmos")
    let address = derive_address_from_mnemonic(&mnemonic, "cosmos")?;
    Ok((mnemonic, address))
}

/// Derive bostrom-compatible address from stored mnemonic
pub fn derive_address() -> Result<String> {
    let mnemonic_phrase = load_mnemonic()?;
    let mnemonic = Mnemonic::from_str(&mnemonic_phrase)?;
    derive_address_from_mnemonic(&mnemonic, "cosmos") // change prefix to bostrom-specific if known
}

/// Helper to derive address
fn derive_address_from_mnemonic(mnemonic: &Mnemonic, bech32_prefix: &str) -> Result<String> {
    let seed = mnemonic.to_seed("");
    let derived = bip32::DerivationPath::from_str("m/44'/118'/0'/0/0")?;
    let child_key = bip32::XPrv::derive_from_path(seed, &derived)?;
    let signing_key = SigningKey::from(child_key);
    let pubkey = signing_key.public_key();
    let account_id = pubkey.account_id(bech32_prefix)?;
    Ok(account_id.to_string())
}

/// Load mnemonic from keyring (never exposed in memory longer than needed)
fn load_mnemonic() -> Result<String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ENTRY)?;
    let phrase = entry.get_password()?;
    Ok(phrase)
}

/// Sign authentication payload (off-chain)
pub async fn sign_payload(payload: &AuthPayload) -> Result<(String, Vec<u8>)> {
    let mnemonic_phrase = load_mnemonic()?;
    let mut phrase_bytes = mnemonic_phrase.into_bytes();
    let mnemonic = Mnemonic::from_str(std::str::from_utf8(&phrase_bytes)?)?;
    phrase_bytes.zeroize();

    let seed = mnemonic.to_seed("");
    let derived = bip32::DerivationPath::from_str("m/44'/118'/0'/0/0")?;
    let child_key = bip32::XPrv::derive_from_path(seed, &derived)?;
    let signing_key = SigningKey::from(child_key);

    let address = derive_address_from_mnemonic(&mnemonic, "cosmos")?;

    // Serialize payload canonically
    let payload_bytes = serde_json::to_vec(payload)?;
    let hash = Sha256::digest(&payload_bytes);

    let signature = signing_key.sign(&hash)?;

    Ok((address, signature.to_vec()))
}

/// Verify signature off-chain and recover address
pub fn verify_payload(payload: &AuthPayload, signature: &[u8], expected_address: &str) -> Result<bool> {
    let payload_bytes = serde_json::to_vec(payload)?;
    let hash = Sha256::digest(&payload_bytes);

    let sig = cosmrs::crypto::secp256k1::Signature::from_slice(signature)?;
    let pubkey = sig.recover_pubkey(&hash)?;
    let recovered = pubkey.account_id("cosmos")?;

    Ok(recovered.to_string() == expected_address)
}
