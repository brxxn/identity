use std::{
  collections::HashMap,
  error::Error,
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};

use base64::prelude::*;
use rand::RngCore;
use rsa::{
  RsaPrivateKey,
  pkcs8::{DecodePrivateKey, EncodePrivateKey},
};

fn generate_rsa_key(out_file: PathBuf) -> Result<RsaPrivateKey, Box<dyn Error>> {
  let mut rng = rand::thread_rng();
  let priv_key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to generate RSA key");

  std::fs::write(out_file, priv_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?)?;

  Ok(priv_key)
}

fn generate_hs256_key(out_file: PathBuf) -> Result<String, Box<dyn Error>> {
  let mut rng = rand::thread_rng();
  let mut key = [0u8; 32];

  rng.fill_bytes(&mut key);
  let base64_encoded_key = BASE64_URL_SAFE.encode(key);
  tracing::info!("Creating hs256 key at {}", out_file.display());
  std::fs::write(out_file, &base64_encoded_key)?;

  Ok(base64_encoded_key)
}

fn read_or_gen_hs256_key(out_file: PathBuf) -> Result<String, Box<dyn Error>> {
  match std::fs::read_to_string(&out_file) {
    Ok(v) => Ok(v.trim().to_string()),
    Err(_) => generate_hs256_key(out_file),
  }
}

pub fn create_keys(key_dir: String) -> Result<crate::AppPrivateKeys, Box<dyn Error>> {
  tracing::info!(
    "Creating key directory and generating new keys... (this invalidated any pre-existing keys!)"
  );
  let key_path = Path::new(&key_dir);
  std::fs::create_dir_all(key_path)?;

  let oidc_key_path = key_path.join("oidc");
  std::fs::create_dir_all(&oidc_key_path)?;
  let timestamp = std::time::SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("time has somehow gone backwards...")
    .as_secs();

  let oidc_jwk = generate_rsa_key(oidc_key_path.join(format!("{}.pem", timestamp)))?;
  let mut oidc_hashmap = HashMap::new();
  oidc_hashmap.insert(timestamp, oidc_jwk);

  Ok(crate::AppPrivateKeys {
    passkey_registration_key: generate_hs256_key(key_path.join("passkey_reg.key"))?,
    passkey_authentication_key: generate_hs256_key(key_path.join("passkey_auth.key"))?,
    oidc_jwt_keys: oidc_hashmap,
    identity_access_jwt_key: generate_hs256_key(key_path.join("identity_access.key"))?,
    identity_refresh_jwt_key: generate_hs256_key(key_path.join("identity_refresh.key"))?,
    registration_jwt_key: generate_hs256_key(key_path.join("registration.key"))?,
  })
}

pub fn load_keys(key_dir: String) -> Result<crate::AppPrivateKeys, Box<dyn Error>> {
  let key_path = Path::new(&key_dir);
  if !std::fs::exists(key_path)? || std::fs::read_dir(key_path)?.next().is_none() {
    tracing::info!("Key path does not exist, generating new keys...");
    return create_keys(key_dir);
  }

  let oidc_key_path = key_path.join("oidc");
  if !std::fs::exists(&oidc_key_path)? {
    panic!(
      "No OIDC path found when loading keys! Generate new keys or add them to /keys/oidc/ to continue"
    );
  }

  let oidc_dir = std::fs::read_dir(&oidc_key_path)?;
  let mut oidc_hashmap = HashMap::new();
  for entry_result in oidc_dir {
    let entry = entry_result?;
    let entry_file_name = entry.file_name();
    let (key, _) = entry_file_name
      .to_str()
      .expect("Invalid filename in OIDC keys")
      .split_once(".")
      .expect("Invalid file in OIDC keys directory!");

    let priv_key_value = std::fs::read_to_string(entry.path())
      .expect("Failed to read OIDC key! Check that each file in key directory is readable!");
    let priv_key = RsaPrivateKey::from_pkcs8_pem(&priv_key_value)
      .expect("PEM-encoded key is invalid and cannot be read!");

    oidc_hashmap.insert(
      key
        .parse::<u64>()
        .expect("Non-integer named OIDC keys are invalid! Please delete keys and regenerate!"),
      priv_key,
    );
  }

  Ok(crate::AppPrivateKeys {
    passkey_registration_key: read_or_gen_hs256_key(key_path.join("passkey_reg.key"))?,
    passkey_authentication_key: read_or_gen_hs256_key(key_path.join("passkey_auth.key"))?,
    oidc_jwt_keys: oidc_hashmap,
    identity_access_jwt_key: read_or_gen_hs256_key(key_path.join("identity_access.key"))?,
    identity_refresh_jwt_key: read_or_gen_hs256_key(key_path.join("identity_refresh.key"))?,
    registration_jwt_key: read_or_gen_hs256_key(key_path.join("registration.key"))?,
  })
}
