use std::collections::HashMap;
use std::time::Duration;

use http::header::AUTHORIZATION;
use jwtk::jwk::RemoteJwksVerifier;
use serde::Deserialize;
use tonic::metadata::MetadataMap;
use tonic::Status;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
struct ExtraClaims {
    #[serde(rename = "urn:zitadel:iam:user:metadata")]
    metadata: HashMap<String, String>,
}

pub fn init_jwks_verifier(
    jwks_host: &str,
    jwks_url: &String,
) -> Result<RemoteJwksVerifier, Box<dyn std::error::Error>> {
    //   adding host header in order to work in private network
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::HOST,
        reqwest::header::HeaderValue::from_str(jwks_host)?,
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(RemoteJwksVerifier::new(
        jwks_url.to_owned(),
        Some(client),
        Duration::from_secs(120),
    ))
}

fn get_token(metadata: &MetadataMap) -> Result<String, Status> {
    metadata
        .get(AUTHORIZATION.as_str())
        .and_then(|v| v.to_str().ok())
        .and_then(|header_value| header_value.split_once(' '))
        .map(|(_, token)| token.to_string())
        .ok_or_else(|| Status::unauthenticated(""))
}

pub async fn get_user_id(
    metadata: &MetadataMap,
    verifier: &RemoteJwksVerifier,
) -> Result<String, Status> {
    let token = get_token(metadata)?;

    verifier
        .verify::<()>(&token)
        .await
        .map_err(|err| Status::unauthenticated(err.to_string()))?
        .claims()
        .sub
        .clone()
        .ok_or_else(|| Status::unauthenticated(""))
}
