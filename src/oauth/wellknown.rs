use axum::{Json, extract::State};
use serde::Serialize;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::jwk::{
  AlgorithmParameters, CommonParameters, Jwk, JwkSet, KeyAlgorithm, RSAKeyParameters,
};
use rsa::{RsaPrivateKey, traits::PublicKeyParts};
use std::collections::HashMap;

use crate::AppState;

#[derive(Serialize, Clone)]
pub struct WellknownClaim {
  pub issuer: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub userinfo_endpoint: String,
  pub jwks_uri: String,
  pub response_types_supported: Vec<&'static str>,
  pub response_modes_supported: Vec<&'static str>,
  pub subject_types_supported: Vec<&'static str>,
  pub id_token_signing_alg_values_supported: Vec<&'static str>,
  pub userinfo_signing_alg_values_supported: Vec<&'static str>
}

fn add_to_issuer(issuer: &String, path: &str) -> String {
  format!("{}{}", issuer, path)
}

pub fn generate_public_jwks(map: HashMap<u64, RsaPrivateKey>) -> JwkSet {
  let keys = map
    .into_iter()
    .map(|(id, private_key)| {
      // Extract public components ONLY
      let n = URL_SAFE_NO_PAD.encode(private_key.n().to_bytes_be());
      let e = URL_SAFE_NO_PAD.encode(private_key.e().to_bytes_be());

      let rsa_params = RSAKeyParameters {
        key_type: jsonwebtoken::jwk::RSAKeyType::RSA,
        n,
        e,
      };

      Jwk {
        common: CommonParameters {
          key_id: Some(id.to_string()), // Unique Key ID
          public_key_use: Some(jsonwebtoken::jwk::PublicKeyUse::Signature), // Purpose: signature
          key_algorithm: Some(KeyAlgorithm::RS256),
          ..Default::default()
        },
        algorithm: AlgorithmParameters::RSA(rsa_params),
      }
    })
    .collect();

  JwkSet { keys }
}

pub async fn openid_configuration(State(state): State<AppState>) -> Json<WellknownClaim> {
  let issuer = state.oidc_issuer_uri;
  Json(WellknownClaim {
    issuer: issuer.clone(),
    authorization_endpoint: add_to_issuer(&issuer, "/oauth/authorize"),
    token_endpoint: add_to_issuer(&issuer, "/v1/oauth/token"),
    userinfo_endpoint: add_to_issuer(&issuer, "/v1/oauth/userinfo"),
    jwks_uri: add_to_issuer(&issuer, "/.well-known/jwks"),
    response_types_supported: vec!["code", "id_token", "id_token token", "code id_token token"],
    response_modes_supported: vec!["query", "fragment"],
    subject_types_supported: vec!["pairwise", "public"],
    id_token_signing_alg_values_supported: vec!["RS256"],
    userinfo_signing_alg_values_supported: vec!["RS256"],
  })
}

pub async fn jwks(State(state): State<AppState>) -> Json<JwkSet> {
  Json(generate_public_jwks(state.private_keys.oidc_jwt_keys))
}
